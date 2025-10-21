use conv::ValueFrom;
use quick_xml::{Reader, events::Event};
use std::collections::HashMap;
use std::error;
use std::fs::File;
use tempfile::tempdir;
use zip::ZipArchive;

const WORLD_META_XML: &str = "world_meta.xml";
const WORLD_XML: &str = "world.xml";

#[derive(Debug, Clone)]
pub struct WorldStats {
    pub date_time: usize,
    pub total_things: usize,
    pub total_rooms: usize,
    pub total_pipe_networks: usize,
    pub total_cable_networks: usize,
    pub total_atmospheres: usize,
    pub total_damage: f64,
    pub players: Vec<String>,
    pub type_map: HashMap<String, usize>,
    pub filename: String,
}

impl WorldStats {
    pub fn new(filename: String) -> Self {
        WorldStats {
            date_time: 0,
            total_things: 0,
            total_rooms: 0,
            total_pipe_networks: 0,
            total_cable_networks: 0,
            total_atmospheres: 0,
            total_damage: 0.0,
            players: Vec::new(),
            type_map: HashMap::new(),
            filename,
        }
    }

    pub fn diff(&self, other: &WorldStats) -> Result<WorldStatsDiff> {
        let mut result = WorldStatsDiff {
            date_time: self.date_time - other.date_time,
            total_things: isize::value_from(self.total_things)?
                - isize::value_from(other.total_things)?,
            total_rooms: isize::value_from(self.total_rooms)?
                - isize::value_from(other.total_rooms)?,
            total_pipe_networks: isize::value_from(self.total_pipe_networks)?
                - isize::value_from(other.total_pipe_networks)?,
            total_cable_networks: isize::value_from(self.total_cable_networks)?
                - isize::value_from(other.total_cable_networks)?,
            total_atmospheres: isize::value_from(self.total_atmospheres)?
                - isize::value_from(other.total_atmospheres)?,
            total_damage: self.total_damage - other.total_damage,
            players: Vec::new(),
            type_map: HashMap::new(),
            new_filename: self.filename.clone(),
            old_filename: other.filename.clone(),
        };
        // Identify new players
        for sp in self.players.iter() {
            if !other.players.contains(sp) {
                result.players.push(sp.clone());
            }
        }
        // Identify types that went down in count
        for (key, value) in self.type_map.iter() {
            if let Some(other_value) = other.type_map.get(key) {
                if *value < *other_value {
                    result.type_map.insert(
                        key.clone(),
                        isize::value_from(*value)? - isize::value_from(*other_value)?,
                    );
                }
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // We're allowing this for now, as it may be used later
pub struct WorldStatsDiff {
    pub date_time: usize,
    pub total_things: isize,
    pub total_rooms: isize,
    pub total_pipe_networks: isize,
    pub total_cable_networks: isize,
    pub total_atmospheres: isize,
    pub total_damage: f64,
    pub players: Vec<String>,
    pub type_map: HashMap<String, isize>,
    pub old_filename: String,
    pub new_filename: String,
}

pub type Result<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

// quick_xml does have an async api, but since we are reading large files, we're generally IO bound which is not threaded anyway,
// so swapping threads is actually slower than scheduling a blocking read.
pub fn parse_save_file(path: &str) -> Result<WorldStats> {
    // Create a temporary directory and extract the zip file
    let temp_dir = tempdir()?;
    ZipArchive::new(File::open(path)?)?.extract(temp_dir.path())?;

    // Prepare a WorldStats struct to hold the statistics
    let mut world_stats = WorldStats::new(path.to_string());

    // Read the world_meta data
    {
        let mut reader = Reader::from_file(temp_dir.path().join(WORLD_META_XML))?;
        let mut buf = Vec::new();
        let mut path_stack = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let text = String::from_utf8(e.local_name().into_inner().to_vec())?;
                    path_stack.push(text);
                }
                Ok(Event::End(_)) => {
                    path_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let text: &str = &e.decode()?;
                    match path_stack.last() {
                        Some(s) if s == "DateTime" => {
                            world_stats.date_time = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfThings" => {
                            world_stats.total_things = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfRooms" => {
                            world_stats.total_rooms = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfPipeNetworks" => {
                            world_stats.total_pipe_networks = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfCableNetworks" => {
                            world_stats.total_cable_networks = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfAtmospheres" => {
                            world_stats.total_atmospheres = text.parse::<usize>()?
                        }
                        None => (),
                        _ => (),
                    }
                }
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
                Ok(Event::Eof) => break,
                _ => (),
            }
            buf.clear();
        }
    }

    // Read the world data
    {
        let mut reader = Reader::from_file(temp_dir.path().join(WORLD_XML))?;
        let mut buf = Vec::new();
        let mut path_stack = Vec::new();
        let mut get_player_name = false;
        let mut damage_state = false;
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let text = String::from_utf8(Vec::from(e.local_name().into_inner()))?;
                    // Look for damage
                    if &text == "DamageState" {
                        damage_state = true; // Set the DamageState flag when we see the tag so we can collect all the damage values
                    }
                    // Look for players
                    let attributes = String::from_utf8(e.attributes_raw().to_vec())?
                        .trim()
                        .to_owned();
                    if !attributes.is_empty() && attributes.starts_with("xsi:type") {
                        // Strip the xsi:type attribute and grab the type name
                        let item_type = attributes.split('"').nth(1);
                        if item_type.is_none() {
                            return Err("Error parsing xsi:type attribute".into());
                        }
                        let item_type = item_type.unwrap().to_owned();
                        // Increment the type count in the type map, or inserting if it doesn't exist already
                        if let Some(count) = world_stats.type_map.get_mut(&item_type) {
                            *count += 1;
                        } else {
                            world_stats.type_map.insert(item_type.clone(), 1);
                        }
                    }
                    if attributes.contains("HumanSaveData") {
                        get_player_name = true; // Set a flag to grab the next "CustomName" tag
                    }
                    path_stack.push(text);
                }
                Ok(Event::End(_)) => {
                    let text = path_stack.pop();
                    if text.is_some() && text.unwrap() == "DamageState" {
                        damage_state = false; // Clear DamageSate flag when we finish parsing that tag
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.decode()?.trim().to_owned();
                    match path_stack.last() {
                        Some(s) if s == "CustomName" && get_player_name => {
                            world_stats.players.push(text);
                            get_player_name = false; // Clear the flag after grabbing the name
                        }
                        Some(_) if damage_state && !text.is_empty() => {
                            world_stats.total_damage += text.parse::<f64>()?;
                        }
                        None => (),
                        _ => (),
                    }
                }
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
                Ok(Event::Eof) => break,
                _ => (),
            }
            buf.clear();
        }
    }

    // Delete the temporary directory
    temp_dir.close()?;
    Ok(world_stats)
}
