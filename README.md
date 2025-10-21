# Stationeers Watchdog

This is a griefing detector for Stationeers. It detects griefing by looking for destructive changes between savefiles and provides easy identification and recovery

**This tool is still under development, and is currently in a Proof-of-Concept version.**

## Usage

Building a Rust crate is left as an exercise to the reader, but the [The Rust Book](https://doc.rust-lang.org/book/) is a great place to start.

To run: `cargo run <autosave_dir>`

Note that this does not currently scan sub-directories, or even take the default Stationeers save directory structure into account. This is planned for the future, and contributions are welcome!
