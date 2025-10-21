mod save_monitor;
use std::{borrow::Cow, env::args};
use std::fs::read_dir;
use colored::Colorize;

use crate::save_monitor::{parse_save_file, WorldStats};

const warn_threshold: isize = -50;
const alarm_threshold: isize = -100;
const damage_warn_threshold: f64 = 500.0;
const damage_alarm_threshold: f64 = 1000.0;
const type_warn_threshold: isize = -10;
const type_alarm_threshold: isize = -20;

fn pretty_print_diff(diff: save_monitor::WorldStatsDiff) -> () {
    let total_atmospheres = match diff.total_atmospheres {
        n if n < alarm_threshold => format!("{:.2}", n).red(),
        n if n < warn_threshold => format!("{:.2}", n).yellow(),
        _ => format!("{:.2}", diff.total_atmospheres).white(),
    };
    let total_damage = match diff.total_damage {
        n if n > damage_alarm_threshold => format!("{:.2}", n).red(),
        n if n > damage_warn_threshold => format!("{:.2}", n).yellow(),
        _ => format!("{:.2}", diff.total_damage).white(),
    };
    let diff_sum: isize = diff.type_map.values().sum();
    let type_map = match diff_sum {
        n if n < type_alarm_threshold => format!("{:?}", diff.type_map).red(),
        n if n < type_warn_threshold => format!("{:?}", diff.type_map).yellow(),
        _ => format!("{:?}", diff.type_map).white(),
    };
    println!("Diff: Atmos: {} Damage: {} Players: [{}]", total_atmospheres, total_damage, diff.players.join(", ").yellow());
    println!("  Types: {}", type_map);

}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Usage: {} <save_directory>", args[0]);
        return;
    }
    let save_dir_path = &args[1];
    let mut saves: Vec<WorldStats> =  Vec::new();
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
                        Err(e) => println!("Error loading save file: {e}")
                    }
                }
            }
            Err(e) => println!("Error reading file: {e}")
        }
    }
    saves.sort_by(|a, b| a.date_time.cmp(&b.date_time));
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
