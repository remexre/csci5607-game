//! The actual rendering code.

use cgmath::{Deg, Matrix4};
use crate::{
    components::{CameraComponent, LocationComponent},
    systems::GuiSystem,
    Model, Vertex, World,
};
use glium::{
    glutin::dpi::LogicalSize,
    index::{NoIndices, PrimitiveType},
    texture::RawImage2d,
    uniforms::{Sampler, SamplerWrapFunction},
    Program, Surface, Texture2d, VertexBuffer,
};
use std::{cell::RefCell, collections::HashMap, ptr::null, rc::Rc, sync::Arc};

/// A graphical component.
pub struct RenderComponent {
    /// The model for the component.
    pub model: Arc<Model>,
}

impl_Component!(RenderComponent);

/// The data required to render a world.
pub struct RenderData {
    /// The clear color.
    pub clear_color: [f32; 4],

    /// The GLSL program.
    pub program: Program,

    /// The dimensions of the window.
    pub(super) dims: LogicalSize,

    /// The projection matrix.
    proj: Matrix4<f32>,

    /// A map from texture images (by address) to textures.
    textures: RefCell<HashMap<*const RawImage2d<'static, u8>, Rc<Texture2d>>>,

    /// A map from models (by address) to VBOs.
    vbos: RefCell<HashMap<*const Model, Rc<VertexBuffer<Vertex>>>>,
}

impl RenderData {
    /// Creates a RenderData with the given clear color and GLSL program.
    pub fn new(clear_color: [f32; 4], program: Program) -> RenderData {
        RenderData {
            clear_color,
            program,
            dims: LogicalSize::new(0.0, 0.0),
            proj: Matrix4::from_scale(0.0),
            textures: RefCell::new(HashMap::new()),
            vbos: RefCell::new(HashMap::new()),
        }
    }
}

impl GuiSystem<RenderData> {
    /// Does the work of rendering a frame.
    pub(super) fn render(&mut self, world: &mut World, frame: &mut impl Surface) {
        let indices = NoIndices(PrimitiveType::TrianglesList);

        let view_mat = match world.iter().next() {
            Some((_, hlist_pat![camera, loc])) => {
                let _: &CameraComponent = camera;
                let loc: &LocationComponent = loc;
                loc.view()
            }
            None => return,
        };

        for (_entity, hlist_pat![render, loc]) in world.iter() {
            let render: &RenderComponent = render;
            let loc: LocationComponent = *loc;

            let (texture, vbo) = self.get_texture_and_vbo(&render.model);

            let uniforms = uniform!{
                ambient: render.model.material.ambient,
                diffuse: render.model.material.diffuse,
                model: Into::<[[f32; 4]; 4]>::into(loc.model()),
                proj: Into::<[[f32; 4]; 4]>::into(self.data.proj),
                tex: Sampler::new(&*texture).wrap_function(SamplerWrapFunction::Repeat),
                textured: render.model.material.texture.is_some(),
                view: Into::<[[f32; 4]; 4]>::into(view_mat),
            };
            frame
                .draw(&*vbo, indices, &self.data.program, &uniforms, &self.params)
                .unwrap()
        }
    }

    /// (Re)-computes the projection matrix.
    pub(super) fn recompute_proj(&mut self) {
        use cgmath::PerspectiveFov;

        let size = self.display.gl_window().get_inner_size().unwrap();
        self.data.dims = size;
        self.data.proj = Matrix4::from(PerspectiveFov {
            fovy: Deg(59.0).into(),
            aspect: (size.width / size.height) as _,
            near: 0.1,
            far: 100.0,
        });
    }

    fn get_texture_and_vbo(&self, model: &Model) -> (Rc<Texture2d>, Rc<VertexBuffer<Vertex>>) {
        let model_ptr = model as _;
        if !self.data.vbos.borrow().contains_key(&model_ptr) {
            let vbo = VertexBuffer::new(&self.display, &model.vertices).unwrap();
            self.data.vbos.borrow_mut().insert(model_ptr, Rc::new(vbo));
        }
        let vbo = self.data.vbos.borrow().get(&model_ptr).unwrap().clone();

        let texture = if let Some(ref texture) = model.material.texture {
            let texture: &RawImage2d<u8> = &*texture;
            let texture_ptr = texture as _;
            if self.data.textures.borrow().contains_key(&texture_ptr) {
                self.data
                    .textures
                    .borrow()
                    .get(&texture_ptr)
                    .unwrap()
                    .clone()
            } else {
                // TODO: The fact that this is necessary feels bug-report-worthy...
                let texture_clone = RawImage2d {
                    data: texture.data.clone(),
                    format: texture.format,
                    height: texture.height,
                    width: texture.width,
                };
                let texture = Rc::new(Texture2d::new(&self.display, texture_clone).unwrap());
                self.data
                    .textures
                    .borrow_mut()
                    .insert(texture_ptr, texture.clone());
                texture
            }
        } else if self.data.textures.borrow().contains_key(&null()) {
            self.data.textures.borrow().get(&null()).unwrap().clone()
        } else {
            let texture =
                Rc::new(Texture2d::new(&self.display, vec![vec![(1.0, 0.0, 1.0, 0.0)]]).unwrap());
            self.data
                .textures
                .borrow_mut()
                .insert(null(), texture.clone());
            texture
        };

        (texture, vbo)
    }
}
