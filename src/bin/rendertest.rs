#[macro_use]
extern crate failure;
#[macro_use]
extern crate frunk;
extern crate game;
extern crate glium;
#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate structopt;

use failure::{Fallible, ResultExt};
use game::{
    util::log_err, GuiSystem, LocationComponent, Model, RenderComponent, RenderData, State,
    SystemStepper, World,
};
use glium::Program;
use std::{process::exit, sync::Arc, time::Instant};
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
    let gui = GuiSystem::new(false)
        .with_context(|err| format_err!("Failed to create GUI system: {}", err))?;
    let program = Program::from_source(
        gui.facade(),
        include_str!("../../maps/main.vert"),
        include_str!("../../maps/main.frag"),
        None,
    )?;
    let gui = gui.add_render_data(RenderData::new([0.0; 4], program));

    let mut world = World::default();
    init_world(&mut world);

    let mut state = State::Playing(world);
    let mut systems = hlist![gui];
    let mut last = Instant::now();
    while !state.should_close() {
        let dt = last.elapsed();
        let dt = dt.subsec_millis() as u64 + 1_000_000 * dt.as_secs();

        systems
            .to_mut()
            .map(SystemStepper::with_args(&mut state, dt));

        last = Instant::now();
    }

    Ok(())
}

fn init_world(world: &mut World) {
    world.new_entity(hlist![
        RenderComponent {
            model: Arc::new(Model::quad(
                (0.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (1.0, 1.0, 0.0),
                (1.0, 0.0, 0.0),
                None
            )),
            scale: 1.0
        },
        LocationComponent(0.0, 0.0, 3.0)
    ]);
}
