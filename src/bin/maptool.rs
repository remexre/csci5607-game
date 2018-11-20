extern crate failure;
extern crate game;
#[macro_use]
extern crate log;
extern crate serde_json;
extern crate stderrlog;
#[macro_use]
extern crate structopt;

use failure::Fallible;
use game::{
    util::{log_err, read_file_and_parse_to},
    Map,
};
use std::{fs::File, io::stdout, path::PathBuf, process::exit};
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

    /// The path of the non-JSON map file to load.
    #[structopt(parse(from_os_str))]
    pub input_path: PathBuf,

    /// The subcommand to run.
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// "Upgrades" from the assignment's format to the JSON format.
    #[structopt(name = "upgrade")]
    Upgrade {
        /// The path of the JSON map file to save to.
        #[structopt(parse(from_os_str))]
        output_path: Option<PathBuf>,

        /// Whether to pretty-print the JSON.
        #[structopt(short = "p", long = "pretty")]
        pretty: bool,
    },
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
    let map: Map = read_file_and_parse_to(options.input_path)?;
    match options.command {
        Command::Upgrade {
            output_path,
            pretty,
        } => {
            if let Some(output_path) = output_path {
                let file = File::open(output_path)?;
                if pretty {
                    serde_json::to_writer_pretty(file, &map)?;
                } else {
                    serde_json::to_writer(file, &map)?;
                }
            } else {
                if pretty {
                    serde_json::to_writer_pretty(stdout(), &map)?;
                } else {
                    serde_json::to_writer(stdout(), &map)?;
                }
            }

            Ok(())
        }
    }
}
