use quick_xml::{events::Event, Reader};
use std::error;
use std::fs::File;
use tempfile::tempdir;
use zip::ZipArchive;

const WORLD_META_XML: &str = "world_meta.xml";
const WORLD_XML: &str = "world.xml";

#[derive(Debug)]
pub struct WorldStats {
    DateTime: usize,
    TotalThings: usize,
    TotalRooms: usize,
    TotalPipeNetworks: usize,
    TotalCableNetworks: usize,
    TotalAtmospheres: usize,
    TotalDamage: usize,
    Players: Vec<String>,
}

impl WorldStats {
    pub fn new() -> Self {
        WorldStats {
            DateTime: 0,
            TotalThings: 0,
            TotalRooms: 0,
            TotalPipeNetworks: 0,
            TotalCableNetworks: 0,
            TotalAtmospheres: 0,
            TotalDamage: 0,
            Players: Vec::new(),
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub fn load_save_file(path: &str) -> Result<WorldStats> {
    // Create a temporary directory and extract the zip file
    let temp_dir = tempdir()?;
    ZipArchive::new(File::open(path)?)?.extract(temp_dir.path())?;

    // Prepare a WorldStats struct to hold the statistics
    let mut world_stats = WorldStats::new();

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
                Ok(Event::End(e)) => {
                    path_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let text: &str = &e.decode()?;
                    match path_stack.last() {
                        Some(s) if s == "DateTime" => {
                            world_stats.DateTime = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfThings" => {
                            world_stats.TotalThings = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfRooms" => {
                            world_stats.TotalRooms = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfPipeNetworks" => {
                            world_stats.TotalPipeNetworks = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfCableNetworks" => {
                            world_stats.TotalCableNetworks = text.parse::<usize>()?
                        }
                        Some(s) if s == "NumberOfAtmospheres" => {
                            world_stats.TotalAtmospheres = text.parse::<usize>()?
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
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let text = String::from_utf8(Vec::from(e.local_name().into_inner()))?;
                    path_stack.push(text);
                    // Look for players
                    let attributes = String::from_utf8(e.attributes_raw().to_vec())?;
                    if attributes.contains("HumanSaveData") {
                        // Set a flag to grab the next "CustomName" tag
                        get_player_name = true;
                    }
                }
                Ok(Event::End(e)) => {
                    path_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let text: &str = &e.decode()?;
                    match path_stack.last() {
                        Some(s) if s == "CustomName" && get_player_name => {
                            world_stats.Players.push(text.to_string());
                            get_player_name = false;
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
