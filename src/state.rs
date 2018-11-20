use crate::{
    util::{load_obj, read_file_and_parse_to},
    Entity, LocationComponent, Map, RenderComponent,
};
use failure::{Fallible, ResultExt};
use frunk::hlist::{HCons, HNil};
use std::{collections::HashMap, path::Path};
use typemap::{Key, ShareMap};

/// The global game state.
pub enum State {
    /// The state of the game while the user is trying to solve.
    Playing(World),

    /// The state of the game after the user has completed the maze.
    Done(World, u64),
}

/// The state of the game world during gameplay.
pub struct World {
    next_entity: Entity,
    components: HashMap<Entity, ShareMap>,
}

impl World {
    /// Loads the assets specified in the map, creating a `World` with them.
    pub fn from_map(map: Map, base_path: impl AsRef<Path>) -> Fallible<World> {
        let base_path = base_path.as_ref();
        let mut world = World {
            next_entity: 0,
            components: HashMap::new(),
        };

        // TODO: map.material_floor
        // TODO: map.material_wall
        // TODO: map.model_character

        // Load the keys.
        let key_model = load_obj(base_path.join(map.model_key))?;
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
    pub fn from_map_file(path: impl AsRef<Path>) -> Fallible<World> {
        let map = read_file_and_parse_to(path.as_ref()).context("While loading map")?;
        let base_path = path.as_ref().parent().unwrap_or_else(|| path.as_ref());
        let world = World::from_map(map, base_path).context("While building world")?;
        Ok(world)
    }

    /// Tries to get the given components for a given entity.
    pub fn get<'a, C: ComponentRefHList<'a>>(&'a mut self, entity: Entity) -> Option<C> {
        self.components
            .get(&entity)
            .and_then(ComponentRefHList::get_from_component_map)
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
