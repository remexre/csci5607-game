extern crate failure;
extern crate game;
#[macro_use]
extern crate log;
extern crate stderrlog;
#[macro_use]
extern crate structopt;

use failure::Fallible;
use game::{
    util::{log_err, read_file_and_parse_to},
    GuiSystem, Map,
};
use std::{path::PathBuf, process::exit};
use structopt::StructOpt;

fn main() {
    let options = Options::from_args();
    options.start_logger();

    if let Err(err) = run(options) {
        log_err(err);
        exit(1);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "::structopt::clap::AppSettings::ColoredHelp"))]
pub struct Options {
    /// Turns off message output.
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,

    /// Increases the verbosity. Default verbosity is errors only.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: usize,

    /// The path of the map file to load.
    #[structopt(parse(from_os_str))]
    pub map_path: PathBuf,
}

impl Options {
    /// Sets up logging as specified by the `-q` and `-v` flags.
    pub fn start_logger(&self) {
        if !self.quiet {
            let r = stderrlog::new().verbosity(self.verbose).init();
            if let Err(err) = r {
                error!("Logging couldn't start: {}", err);
            }
        }
    }
}

fn run(options: Options) -> Fallible<()> {
    let map: Map = read_file_and_parse_to(options.map_path)?;

    let mut gui_system = GuiSystem::new()?;
    loop {
        gui_system.step();
    }
}
