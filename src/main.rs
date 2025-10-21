mod save_monitor;
use colored::Colorize;
use std::env::args;
use std::fs::read_dir;

use crate::save_monitor::{parse_save_file, WorldStats};

const ATMOS_WARN_THRESHOLD: isize = -50;
const ATMOS_ALARM_THRESHOLD: isize = -100;
const DAMAGE_WARN_THRESHOLD: f64 = 500.0;
const DAMAGE_ALARM_THRESHOLD: f64 = 1000.0;
const TYPE_WARN_THRESHOLD: isize = -10;
const TYPE_ALARM_THRESHOLD: isize = -20;

fn pretty_print_diff(diff: save_monitor::WorldStatsDiff) {
    let total_atmospheres = match diff.total_atmospheres {
        n if n < ATMOS_ALARM_THRESHOLD => format!("{n:.2}").red(),
        n if n < ATMOS_WARN_THRESHOLD => format!("{n:.2}").yellow(),
        _ => format!("{:.2}", diff.total_atmospheres).white(),
    };
    let total_damage = match diff.total_damage {
        n if n > DAMAGE_ALARM_THRESHOLD => format!("{n:.2}").red(),
        n if n > DAMAGE_WARN_THRESHOLD => format!("{n:.2}").yellow(),
        _ => format!("{:.2}", diff.total_damage).white(),
    };
    let diff_sum: isize = diff.type_map.values().sum();
    let type_map = match diff_sum {
        n if n < TYPE_ALARM_THRESHOLD => format!("{:?}", diff.type_map).red(),
        n if n < TYPE_WARN_THRESHOLD => format!("{:?}", diff.type_map).yellow(),
        _ => format!("{:?}", diff.type_map).white(),
    };
    println!(
        "Diff: Atmos: {} Damage: {} Players: [{}]",
        total_atmospheres,
        total_damage,
        diff.players.join(", ").yellow()
    );
    println!("  Types: {type_map}");
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Usage: {} <save_directory>", args[0]);
        return;
    }
    let save_dir_path = &args[1];
    let mut saves: Vec<WorldStats> = Vec::new();
    let save_dir = read_dir(save_dir_path);
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
                        Err(e) => println!("Error loading save file: {e}"),
                    }
                }
            }
            Err(e) => println!("Error reading file: {e}"),
        }
    }
    saves.sort_by(|a, b| a.date_time.cmp(&b.date_time));
    let mut last_save: Option<&WorldStats> = None;
    for cur_save in saves.iter() {
        if last_save.is_some() {
            match cur_save.diff(last_save.unwrap()) {
                Ok(diff) => pretty_print_diff(diff),
                Err(e) => println!("Diff Error: {e}"),
            }
        }
        println!("{}", cur_save.filename);
        last_save = Some(cur_save);
    }
}
