use crate::{
    gui::RenderData,
    util::{read_file, read_file_and_parse_to},
    Entity, LocationComponent, Map, Model, RenderComponent,
};
use failure::{Fallible, ResultExt};
use frunk::hlist::{HCons, HNil};
use glium::{backend::Facade, Program};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, path::Path};
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
    next_entity: Entity,
    components: HashMap<Entity, ShareMap>,
}

impl World {
    /// Loads the assets specified in the map, creating a `World` with them.
    pub fn from_map(
        map: Map,
        base_path: impl AsRef<Path>,
        facade: &impl Facade,
    ) -> Fallible<(RenderData, World)> {
        let base_path = base_path.as_ref();

        let mut world = World {
            next_entity: 0,
            components: HashMap::new(),
        };

        // TODO: map.material_floor
        // TODO: map.material_wall
        // TODO: map.model_character

        // Load the keys.
        let key_model = Model::load_obj(base_path.join(&map.model_key))?;
        for (x, y, ch) in map.keys {
            world.new_entity(hlist![
                RenderComponent {
                    model: key_model.clone()
                },
                LocationComponent(x as f32 + 0.5, y as f32 + 0.5, 0.25)
            ]);
        }

        let render_data = RenderData {
            clear_color: map.clear_color,
            program: Program::from_source(
                facade,
                &read_file(base_path.join(&map.shader_vert))?,
                &read_file(base_path.join(&map.shader_frag))?,
                None,
            )?,
        };
        Ok((render_data, world))
    }

    /// Loads the world from the map whose file path is given.
    pub fn from_map_file(
        path: impl AsRef<Path>,
        facade: &impl Facade,
    ) -> Fallible<(RenderData, World)> {
        let map = read_file_and_parse_to(path.as_ref()).context("While loading map")?;
        let base_path = path.as_ref().parent().unwrap_or_else(|| path.as_ref());
        World::from_map(map, base_path, facade)
            .context("While building world")
            .map_err(From::from)
    }

    /// Creates a `World` for examples. Don't actually use this!
    pub fn example() -> World {
        // use glium::{backend::glutin::headless::Headless, glutin::HeadlessRendererBuilder};

        // let ctx = HeadlessRendererBuilder::new(1, 1).build().unwrap();
        // let facade = Headless::new(ctx).unwrap();
        World {
            /*
            clear_color: [0.0; 4],
            program: Program::from_source(
                &facade,
                "void main(){gl_Position=vec4(0);}",
                "void main(){}",
                None,
            ).unwrap(),
            */
            next_entity: 0,
            components: HashMap::new(),
        }
    }

    /// Tries to get the given components for a given entity.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use] extern crate frunk;
    /// # extern crate game;
    /// # #[macro_use] extern crate typemap;
    /// # fn main() {
    /// #[derive(Debug, PartialEq)]
    /// struct FooComponent(&'static str);
    /// impl typemap::Key for FooComponent { type Value = FooComponent; }
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct BarComponent(usize);
    /// impl typemap::Key for BarComponent { type Value = BarComponent; }
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct BazComponent;
    /// impl typemap::Key for BazComponent { type Value = BazComponent; }
    ///
    /// let mut world = game::World::example();
    /// let me = world.new_entity(hlist![FooComponent("hello"), BarComponent(42)]);
    ///
    /// assert_eq!(world.get(me), Some(hlist![&FooComponent("hello")]));
    /// assert_eq!(world.get(me), Some(hlist![&BarComponent(42)]));
    /// assert_eq!(world.get(me), Some(hlist![
    ///     &FooComponent("hello"),
    ///     &BarComponent(42),
    /// ]));
    /// assert_eq!(world.get::<Hlist![&BazComponent]>(me), None);
    /// # }
    /// ```
    pub fn get<'a, C: ComponentRefHList<'a>>(&'a self, entity: Entity) -> Option<C> {
        self.components
            .get(&entity)
            .and_then(ComponentRefHList::get_from_component_map)
    }

    /// Tries to get a single component, mutably, for a given entity.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use] extern crate frunk;
    /// # extern crate game;
    /// # #[macro_use] extern crate typemap;
    /// # fn main() {
    /// #[derive(Debug, PartialEq)]
    /// struct FooComponent(&'static str);
    /// impl typemap::Key for FooComponent { type Value = FooComponent; }
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct BarComponent(usize);
    /// impl typemap::Key for BarComponent { type Value = BarComponent; }
    ///
    /// let mut world = game::World::example();
    /// let me = world.new_entity(hlist![FooComponent("hello")]);
    ///
    /// assert_eq!(world.get_mut(me), Some(&mut FooComponent("hello")));
    /// assert_eq!(world.get_mut::<BarComponent>(me), None);
    /// # }
    /// ```
    pub fn get_mut<T>(&mut self, entity: Entity) -> Option<&mut T>
    where
        T: Key<Value = T> + Send + Sync,
    {
        self.components
            .get_mut(&entity)
            .and_then(ShareMap::get_mut::<T>)
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
    /// #[derive(Debug)]
    /// struct BarComponent(usize);
    /// impl typemap::Key for BarComponent { type Value = BarComponent; }
    ///
    /// let mut world = game::World::example();
    /// world.new_entity(hlist![FooComponent]);
    /// world.new_entity(hlist![FooComponent, BarComponent(42)]);
    /// for (e, hlist_pat![foo, bar]) in world.iter() {
    ///     println!("Entity: {:?}", e);
    ///     println!("Foo: {:?}", foo as &FooComponent);
    ///     println!("Bar: {:?}", bar as &BarComponent);
    /// }
    /// # }
    /// ```
    pub fn iter<'a, C>(&'a self) -> impl 'a + Iterator<Item = (Entity, C)>
    where
        C: 'a + ComponentRefHList<'a>,
    {
        self.components
            .keys()
            .cloned()
            .filter_map(move |entity| self.get(entity).map(|cs| (entity, cs)))
    }

    /// Creates a new entity with the given components.
    pub fn new_entity<C: ComponentHList>(&mut self, components: C) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        let mut map = ShareMap::custom();
        components.add_to_component_map(&mut map);
        self.components.insert(entity, map);
        entity
    }

    /// Iterates in parallel over entities which have all the given components.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use] extern crate frunk;
    /// # extern crate game;
    /// # #[macro_use] extern crate typemap;
    /// # extern crate rayon;
    /// # use rayon::iter::ParallelIterator;
    /// # fn main() {
    /// #[derive(Debug)]
    /// struct FooComponent;
    /// impl typemap::Key for FooComponent { type Value = FooComponent; }
    ///
    /// #[derive(Debug)]
    /// struct BarComponent(usize);
    /// impl typemap::Key for BarComponent { type Value = BarComponent; }
    ///
    /// let mut world = game::World::example();
    /// world.new_entity(hlist![FooComponent]);
    /// world.new_entity(hlist![FooComponent, BarComponent(42)]);
    /// world.par_iter().for_each(|(e, hlist_pat![foo, bar])| {
    ///     println!("Entity: {:?}", e);
    ///     println!("Foo: {:?}", foo as &FooComponent);
    ///     println!("Bar: {:?}", bar as &BarComponent);
    /// });
    /// # }
    /// ```
    pub fn par_iter<'a, C>(&'a self) -> impl 'a + ParallelIterator<Item = (Entity, C)>
    where
        C: 'a + ComponentRefHList<'a> + Send,
    {
        self.components
            .par_iter()
            .map(|(&k, _)| k)
            .filter_map(move |entity| self.get(entity).map(|cs| (entity, cs)))
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
