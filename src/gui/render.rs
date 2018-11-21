//! The actual rendering code.

use cgmath::{Deg, Matrix4, Point3, Vector3};
use crate::{GuiSystem, LocationComponent, Model, Vertex, World};
use glium::{
    draw_parameters::DrawParameters,
    index::{NoIndices, PrimitiveType},
    Program, Surface, Texture2d, VertexBuffer,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

/// A graphical component.
pub struct RenderComponent {
    /// The model for the component.
    pub model: Arc<Model>,

    /// The amount to scale by.
    pub scale: f32,
}

impl_Component!(RenderComponent);

/// The data required to render a world.
pub struct RenderData {
    /// The clear color.
    pub clear_color: [f32; 4],

    /// The GLSL program.
    pub program: Program,

    /// The projection matrix.
    proj: Matrix4<f32>,

    /// A map from models (by address) to Texture-VBO pairs.
    textures_vbos: RefCell<HashMap<*const Model, Rc<(Texture2d, VertexBuffer<Vertex>)>>>,
}

impl RenderData {
    /// Creates a RenderData with the given clear color and GLSL program.
    pub fn new(clear_color: [f32; 4], program: Program) -> RenderData {
        RenderData {
            clear_color,
            program,
            proj: Matrix4::from_scale(0.0),
            textures_vbos: RefCell::new(HashMap::new()),
        }
    }
}

impl GuiSystem<RenderData> {
    /// Does the work of rendering a frame.
    pub(super) fn render(&mut self, world: &mut World, frame: &mut impl Surface) {
        let indices = NoIndices(PrimitiveType::TrianglesList);
        let draw_params = DrawParameters::default();

        let view_mat = Matrix4::look_at(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(5.0, 5.0, 5.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        for (_entity, hlist_pat![render, pos]) in world.iter() {
            let render: &RenderComponent = render;
            let LocationComponent(x, y, z) = *pos;

            let (texture, vbo) = &*self.get_vbo(&render.model);

            let model_mat = Matrix4::from_translation(Vector3 { x, y, z })
                // * rotation_matrix
                * Matrix4::from_scale(render.scale);

            let uniforms = uniform!{
                model: Into::<[[f32; 4]; 4]>::into(model_mat),
                proj: Into::<[[f32; 4]; 4]>::into(self.data.proj),
                view: Into::<[[f32; 4]; 4]>::into(view_mat),
            };
            frame
                .draw(vbo, indices, &self.data.program, &uniforms, &draw_params)
                .unwrap()
        }
    }

    /// (Re)-computes the projection matrix.
    pub(super) fn recompute_proj(&mut self) {
        use cgmath::PerspectiveFov;

        let size = self.display.gl_window().get_inner_size().unwrap();
        self.data.proj = Matrix4::from(PerspectiveFov {
            fovy: Deg(70.0).into(),
            aspect: (size.width / size.height) as _,
            near: 0.1,
            far: 5.0,
        });
    }

    fn get_vbo(&self, model: &Model) -> Rc<(Texture2d, VertexBuffer<Vertex>)> {
        let model_ptr = model as _;
        if !self.data.textures_vbos.borrow().contains_key(&model_ptr) {
            let texture = if let Some(texture) = model.texture.as_ref() {
                // Texture2d::new(&self.display, texture)
                unimplemented!();
            } else {
                Texture2d::new(&self.display, vec![vec![(1.0, 0.0, 1.0, 0.0)]])
            }.unwrap();
            let vbo = VertexBuffer::new(&self.display, &model.vertices).unwrap();
            self.data
                .textures_vbos
                .borrow_mut()
                .insert(model_ptr, Rc::new((texture, vbo)));
        }
        self.data
            .textures_vbos
            .borrow()
            .get(&model_ptr)
            .unwrap()
            .clone()
    }
}
