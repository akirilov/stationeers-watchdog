mod save_monitor;
use std::env::args;

use crate::save_monitor::load_save_file;

fn main() {
    let result = load_save_file(args().nth(1).unwrap().as_str());
    println!("{result:?}");
}
