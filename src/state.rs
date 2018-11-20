use crate::{util::read_file_and_parse_to, Entity, LocationComponent, Map, Model, RenderComponent};
use failure::{Fallible, ResultExt};
use frunk::hlist::{HCons, HNil};
use glium::{backend::Facade, Program};
use std::{collections::HashMap, fs::read_to_string, marker::PhantomData, path::Path};
use typemap::{Key, ShareMap};

/// The global game state.
pub enum State {
    /// The state of the game while the user is trying to solve.
    Playing(World),

    /// The state of the game after the user has completed the maze.
    Done(World, u64),

    /// The state that represents a requested close.
    Close,
}

impl State {
    /// Returns whether the state indicates that closing should occur.
    pub fn should_close(&self) -> bool {
        match *self {
            State::Close => true,
            _ => false,
        }
    }
}

/// The state of the game world during gameplay.
pub struct World {
    /// The color to clear the display with each frame.
    pub clear_color: [f32; 4],

    /// The GLSL program used for rendering.
    pub program: Program,

    next_entity: Entity,
    components: HashMap<Entity, ShareMap>,
}

impl World {
    /// Loads the assets specified in the map, creating a `World` with them.
    pub fn from_map(
        map: Map,
        base_path: impl AsRef<Path>,
        facade: &impl Facade,
    ) -> Fallible<World> {
        let base_path = base_path.as_ref();

        let program = Program::from_source(
            facade,
            &read_to_string(base_path.join(&map.shader_vert)).with_context(|_| {
                format_err!(
                    "Failed to read vertex shader ({})",
                    map.shader_vert.display()
                )
            })?,
            &read_to_string(base_path.join(&map.shader_frag)).with_context(|_| {
                format_err!(
                    "Failed to read fragment shader ({})",
                    map.shader_frag.display()
                )
            })?,
            None,
        )?;
        let mut world = World {
            clear_color: map.clear_color,
            program,
            next_entity: 0,
            components: HashMap::new(),
        };

        // TODO: map.material_floor
        // TODO: map.material_wall
        // TODO: map.model_character

        // Load the keys.
        let key_model = Model::load_obj(base_path.join(map.model_key))?;
        for (x, y, ch) in map.keys {
            world.new_entity(hlist![
                RenderComponent {
                    model: key_model.clone()
                },
                LocationComponent(x as f32 + 0.5, y as f32 + 0.5, 0.25)
            ]);
        }

        Ok(world)
    }

    /// Loads the world from the map whose file path is given.
    pub fn from_map_file(path: impl AsRef<Path>, facade: &impl Facade) -> Fallible<World> {
        let map = read_file_and_parse_to(path.as_ref()).context("While loading map")?;
        let base_path = path.as_ref().parent().unwrap_or_else(|| path.as_ref());
        let world = World::from_map(map, base_path, facade).context("While building world")?;
        Ok(world)
    }

    /// Creates a `World` for examples. Don't actually use this!
    pub fn example() -> World {
        use glium::{backend::glutin::headless::Headless, glutin::HeadlessRendererBuilder};

        let ctx = HeadlessRendererBuilder::new(1, 1).build().unwrap();
        let facade = Headless::new(ctx).unwrap();
        World {
            clear_color: [0.0; 4],
            program: Program::from_source(
                &facade,
                "void main(){gl_Position=vec4(0);}",
                "void main(){}",
                None,
            ).unwrap(),
            next_entity: 0,
            components: HashMap::new(),
        }
    }

    /// Tries to get the given components for a given entity.
    pub fn get<'a, C: ComponentRefHList<'a>>(&'a self, entity: Entity) -> Option<C> {
        self.components
            .get(&entity)
            .and_then(ComponentRefHList::get_from_component_map)
    }

    /// Iterates over entities which have all the given components.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use] extern crate frunk;
    /// # extern crate game;
    /// # #[macro_use] extern crate typemap;
    /// # fn main() {
    /// #[derive(Debug)]
    /// struct FooComponent;
    /// impl typemap::Key for FooComponent { type Value = FooComponent; }
    ///
    /// let world = game::World::example();
    /// for (e, hlist_pat![foo]) in world.iter() {
    ///     println!("Entity: {:?}", e);
    ///     println!("Foo: {:?}", foo as &FooComponent);
    /// }
    /// # }
    /// ```
    pub fn iter<'a, C>(&'a self) -> impl 'a + Iterator<Item = (Entity, C)>
    where
        C: 'a + ComponentRefHList<'a>,
    {
        Iter(self, self.components.keys(), PhantomData)
    }

    /// Creates a new entity with the given components.
    pub fn new_entity<C: ComponentHList>(&mut self, components: C) {
        let entity = self.next_entity;
        self.next_entity += 1;
        let mut map = ShareMap::custom();
        components.add_to_component_map(&mut map);
        self.components.insert(entity, map);
    }
}

/// A trait for an HList containing only components (i.e. types that
/// `impl typemap::Key<Value = Self>`).
pub trait ComponentHList {
    fn add_to_component_map(self, component_map: &mut ShareMap);
}

impl<H, T> ComponentHList for HCons<H, T>
where
    H: Key<Value = H> + Send + Sync,
    T: ComponentHList,
{
    fn add_to_component_map(self, component_map: &mut ShareMap) {
        component_map.insert::<H>(self.head);
        self.tail.add_to_component_map(component_map);
    }
}

impl ComponentHList for HNil {
    fn add_to_component_map(self, _: &mut ShareMap) {}
}

/// A trait for an HList containing only references to components.
pub trait ComponentRefHList<'a>: Sized {
    fn get_from_component_map(component_map: &'a ShareMap) -> Option<Self>;
}

impl<'a, H, T> ComponentRefHList<'a> for HCons<&'a H, T>
where
    H: Key<Value = H> + Send + Sync,
    T: ComponentRefHList<'a>,
{
    fn get_from_component_map(component_map: &'a ShareMap) -> Option<Self> {
        let head = component_map.get::<H>()?;
        let tail = T::get_from_component_map(component_map)?;
        Some(HCons { head, tail })
    }
}

impl<'a> ComponentRefHList<'a> for HNil {
    fn get_from_component_map(_: &'a ShareMap) -> Option<Self> {
        Some(HNil)
    }
}

/// An iterator over the components. See `World::iter`.
struct Iter<'a, C>(
    &'a World,
    std::collections::hash_map::Keys<'a, usize, ShareMap>,
    PhantomData<C>,
);

impl<'a, C: ComponentRefHList<'a>> Iterator for Iter<'a, C> {
    type Item = (Entity, C);
    fn next(&mut self) -> Option<(Entity, C)> {
        loop {
            let entity = self.1.next()?.clone();
            if let Some(cs) = self.0.get(entity) {
                break Some((entity, cs));
            }
        }
    }
}
