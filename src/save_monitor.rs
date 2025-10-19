use quick_xml::{events::Event, Reader};
use std::cmp::Ordering;
use std::error;
use std::fs::File;
use tempfile::tempdir;
use zip::ZipArchive;
use conv::ValueFrom;

const WORLD_META_XML: &str = "world_meta.xml";
const WORLD_XML: &str = "world.xml";

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct WorldStats {
    pub date_time: usize,
    pub total_things: usize,
    pub total_rooms: usize,
    pub total_pipe_networks: usize,
    pub total_cable_networks: usize,
    pub total_atmospheres: usize,
    pub total_damage: f64,
    pub players: Vec<String>,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct WorldStatsDiff {
    pub date_time: usize,
    pub total_things: f64,
    pub total_rooms: f64,
    pub total_pipe_networks: f64,
    pub total_cable_networks: f64,
    pub total_atmospheres: f64,
    pub total_damage: f64,
    pub players: Vec<String>,
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
            filename: filename,
        }
    }

    pub fn diff(&self, other: &WorldStats) -> Result<WorldStatsDiff> {
        let mut players_diff = Vec::new();
        for sp in self.players.iter() {
            if !other.players.contains(sp) {
                players_diff.push(sp.clone());
            }
        }
        Ok(WorldStatsDiff {
            date_time: self.date_time - other.date_time,
            total_things: (f64::value_from(self.total_things)? - f64::value_from(other.total_things)?) / f64::value_from(self.total_things)?,
            total_rooms: (f64::value_from(self.total_rooms)? - f64::value_from(other.total_rooms)?) / f64::value_from(self.total_rooms)?,
            total_pipe_networks: (f64::value_from(self.total_pipe_networks)? - f64::value_from(other.total_pipe_networks)?) / f64::value_from(self.total_pipe_networks)?,
            total_cable_networks: (f64::value_from(self.total_cable_networks)? - f64::value_from(other.total_cable_networks)?) / f64::value_from(self.total_cable_networks)?,
            total_atmospheres: (f64::value_from(self.total_atmospheres)? - f64::value_from(other.total_atmospheres)?) / f64::value_from(self.total_atmospheres)?,
            total_damage: (self.total_damage - other.total_damage) / self.total_damage,
            players: players_diff
        })
    }
}

impl Eq for WorldStats {}

impl Ord for WorldStats {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date_time.cmp(&other.date_time)
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

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
                        damage_state = true;
                    }
                    // Look for players
                    let attributes = String::from_utf8(e.attributes_raw().to_vec())?;
                    if attributes.contains("HumanSaveData") {
                        get_player_name = true; // Set a flag to grab the next "CustomName" tag
                    }
                    path_stack.push(text);
                }
                Ok(Event::End(e)) => {
                    let text = path_stack.pop();
                    if text.is_some() && text.unwrap() == "DamageState" {
                        damage_state = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.decode()?.trim().to_owned();
                    match path_stack.last() {
                        Some(s) if s == "CustomName" && get_player_name => {
                            world_stats.players.push(text.into());
                            get_player_name = false; // Clear the flag after grabbing the name
                        }
                        Some(s) if damage_state && !text.is_empty()=> {
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
