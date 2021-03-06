#[macro_use]
extern crate failure;
#[macro_use]
extern crate frunk;
extern crate game;
#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate structopt;

use failure::{Fallible, ResultExt};
use game::{
    systems::{
        GuiSystem, HoldSystem, SinkingDoorSystem, SnagSystem, SpinningKeySystem,
        TheFloorIsLavaSystem, UnlockSystem, WinSystem,
    },
    util::log_err,
    State, SystemStepper, World,
};
use std::{path::PathBuf, process::exit, time::Instant};
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

    /// Disable mouse grabbing.
    #[structopt(long = "no-grab-mouse")]
    pub no_grab_mouse: bool,
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
    let (controls, gui) = GuiSystem::new(!options.no_grab_mouse)
        .with_context(|err| format_err!("Failed to create GUI system: {}", err))?;

    let (render_data, world) = World::from_map_file(options.map_path, gui.facade())?;
    let mut state = State::Playing(world);

    let mut systems = hlist![
        controls,
        gui.add_render_data(render_data),
        HoldSystem,
        SinkingDoorSystem,
        SnagSystem,
        SpinningKeySystem,
        TheFloorIsLavaSystem,
        UnlockSystem,
        WinSystem,
    ];
    let mut last = Instant::now();
    while !state.should_close() {
        let dt = last.elapsed();
        last = Instant::now();
        let dt = dt.subsec_millis() as u64 + 1_000_000 * dt.as_secs();

        systems
            .to_mut()
            .map(SystemStepper::with_args(&mut state, dt));
    }

    Ok(())
}
