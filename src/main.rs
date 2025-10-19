mod save_monitor;
use std::{borrow::Cow, env::args};
use std::fs::read_dir;
use colored::Colorize;

use crate::save_monitor::{parse_save_file, WorldStats};

const save_dir_path: &str = "example";
const warn_threshold: f64 = -5.0;
const alarm_threshold: f64 = -10.0;

fn main() {
    let mut saves: Vec<WorldStats> =  Vec::new();
    let save_dir =read_dir(save_dir_path);
    if save_dir.is_err() {
        println!("Error reading save directory");
        return;
    }
    for entry in save_dir.unwrap() {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|f| f == "save") {
                    match parse_save_file(path.to_string_lossy().as_ref()) {
                        Ok(result) => saves.push(result),
                        Err(e) => println!("Error loading save file: {e}")
                    }
                }
            }
            Err(e) => println!("Error reading file: {e}")
        }
    }
    saves.sort();
    let mut last_save: Option<&WorldStats> = None;
    for cur_save in saves.iter() {
        if last_save.is_some() {
            match cur_save.diff(last_save.unwrap()) {
                Ok(diff) => {
                    pretty_print_diff(diff);
                }
                Err(e) => println!("Diff Error: {e}")
            }
        }
        println!("{}", cur_save.filename);
        last_save = Some(&cur_save);
    }

}

fn pretty_print_diff(diff: save_monitor::WorldStatsDiff) -> () {
    let total_things = match diff.total_things {
        n if n < alarm_threshold => format!("{:.2}", n).red(),
        n if n < warn_threshold => format!("{:.2}", n).yellow(),
        n => format!("{:.2}", n).white(),
    };
    let total_rooms = match diff.total_rooms {
        n if n < alarm_threshold => format!("{:.2}", n).red(),
        n if n < warn_threshold => format!("{:.2}", n).yellow(),
        n => format!("{:.2}", n).white(),
    };
    let total_pipe_networks = match diff.total_pipe_networks {
        n if n < alarm_threshold => format!("{:.2}", n).red(),
        n if n < warn_threshold => format!("{:.2}", n).yellow(),
        _ => format!("{:.2}", diff.total_pipe_networks).white(),
    };
    let total_cable_networks = match diff.total_cable_networks {
        n if n < alarm_threshold => format!("{:.2}", n).red(),
        n if n < warn_threshold => format!("{:.2}", n).yellow(),
        _ => format!("{:.2}", diff.total_cable_networks).white(),
    };
    let total_atmospheres = match diff.total_atmospheres {
        n if n < alarm_threshold => format!("{:.2}", n).red(),
        n if n < warn_threshold => format!("{:.2}", n).yellow(),
        _ => format!("{:.2}", diff.total_atmospheres).white(),
    };
    let total_damage = match diff.total_damage {
        n if n > (-1.0 * alarm_threshold) => format!("{:.2}", n).red(),
        n if n > (-1.0 * warn_threshold) => format!("{:.2}", n).yellow(),
        _ => format!("{:.2}", diff.total_damage).white(),
    };
    println!("Diff: Things: {}% Rooms: {}% PipeNets: {}% CableNets: {}% Atmos: {}% Damage: {}% Players: [{}]", total_things, total_rooms, total_pipe_networks, total_cable_networks, total_atmospheres, total_damage, diff.players.join(", ").yellow());

}
