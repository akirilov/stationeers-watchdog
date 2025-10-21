mod save_monitor;
use colored::Colorize;
use std::env::args;
use std::fs::read_dir;
use tokio::{self, task::spawn};

use crate::save_monitor::{WorldStats, WorldStatsDiff, parse_save_file};

const ATMOS_WARN_THRESHOLD: isize = -50;
const ATMOS_ALARM_THRESHOLD: isize = -100;
const DAMAGE_WARN_THRESHOLD: f64 = 500.0;
const DAMAGE_ALARM_THRESHOLD: f64 = 1000.0;
const TYPE_WARN_THRESHOLD: isize = -10;
const TYPE_ALARM_THRESHOLD: isize = -20;
const ALERT_THRESHOLD: u8 = 4;

fn pretty_print_diff(diff: WorldStatsDiff) {
    // Check each tracked stat in the diff again the thresholds, calculate the severity,
    // and format the color
    let mut alerts = 0;
    let total_atmospheres = match diff.total_atmospheres {
        n if n < ATMOS_ALARM_THRESHOLD => {
            alerts += 2;
            format!("{n:.2}").red()
        }
        n if n < ATMOS_WARN_THRESHOLD => {
            alerts += 1;
            format!("{n:.2}").yellow()
        }
        _ => format!("{:.2}", diff.total_atmospheres).white(),
    };
    let total_damage = match diff.total_damage {
        n if n > DAMAGE_ALARM_THRESHOLD => {
            alerts += 2;
            format!("{n:.2}").red()
        }
        n if n > DAMAGE_WARN_THRESHOLD => {
            alerts += 1;
            format!("{n:.2}").yellow()
        }
        _ => format!("{:.2}", diff.total_damage).white(),
    };
    let type_map = match diff.type_map.values().sum::<isize>() {
        n if n < TYPE_ALARM_THRESHOLD => {
            alerts += 2;
            format!("{:?}", diff.type_map).red()
        }
        n if n < TYPE_WARN_THRESHOLD => {
            alerts += 1;
            format!("{:?}", diff.type_map).yellow()
        }
        _ => format!("{:?}", diff.type_map).white(),
    };
    // If we pass the cumulative threshold for the states, print the diff
    // Always print if there's a new player so we can corellate to who the
    // last player joining is in case a save triggered after they joined
    // but before they started griefing
    if !diff.players.is_empty() || alerts > ALERT_THRESHOLD {
        println!("{} -> {}", diff.old_filename, diff.new_filename);
        println!(
            "  Diff: Atmos: {} Damage: {} Players: [{}]",
            total_atmospheres,
            total_damage,
            diff.players.join(", ").yellow()
        );
        println!("  Types: {type_map}");
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Usage: {} <save_directory>", args[0]);
        return;
    }
    // Enumerate all files in the save directory
    let save_dir_path = &args[1];
    let mut saves: Vec<WorldStats> = Vec::new();
    let mut save_handles = Vec::new();
    let save_dir = read_dir(save_dir_path);
    if save_dir.is_err() {
        println!("Error reading save directory");
        return;
    }
    // Spawn a task for each save file
    for entry in save_dir.unwrap() {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|f| f == "save") {
                    save_handles.push(spawn(async move {
                        parse_save_file(path.to_string_lossy().as_ref())
                    }))
                }
            }
            Err(e) => println!("Error reading file: {e}"),
        }
    }
    // Await all the tasks
    for handle in save_handles {
        match handle.await {
            Ok(result) => match result {
                Ok(result) => saves.push(result),
                Err(e) => println!("Error loading save file: {e}"),
            },
            Err(e) => println!("Error awaiting save file: {e}"),
        }
    }
    // Sort the saves by timestamp and generate and print the diffs
    saves.sort_by(|a, b| a.date_time.cmp(&b.date_time));
    let mut last_save: Option<&WorldStats> = None;
    for cur_save in saves.iter() {
        if last_save.is_some() {
            match cur_save.diff(last_save.unwrap()) {
                Ok(diff) => pretty_print_diff(diff),
                Err(e) => println!("Diff Error: {e}"),
            }
        }
        last_save = Some(cur_save);
    }
}
