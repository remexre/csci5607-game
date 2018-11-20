/// The global game state.
#[derive(Debug)]
pub enum State {
    /// The state of the game while the user is trying to solve.
    Playing(World),

    /// The state of the game after the user has completed the maze.
    Done(u64),
}

/// The state of the game world during gameplay.
#[derive(Debug)]
pub struct World {
    /// The graphics components.
    pub c_gfx: Vec<()>,
}
