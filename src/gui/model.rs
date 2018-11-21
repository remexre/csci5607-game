use cgmath::Vector3;
use failure::{Fallible, ResultExt};
use obj::{Obj, SimplePolygon};
use std::{
    collections::HashMap,
    fmt::Write,
    fs::canonicalize,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
};

/// A single vertex.
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    /// The spatial position of the vertex.
    pub xyz: [f32; 3],

    /// The normal vector at the vertex.
    pub normal: [f32; 3],

    /// The texture coordinates at the vertex.
    pub uv: [f32; 2],
}

impl Vertex {
    /// Creates a Vertex.
    pub fn new(
        xyz: impl Into<[f32; 3]>,
        normal: impl Into<[f32; 3]>,
        uv: impl Into<[f32; 2]>,
    ) -> Vertex {
        Vertex {
            xyz: xyz.into(),
            normal: normal.into(),
            uv: uv.into(),
        }
    }
}

implement_vertex!(Vertex, xyz, normal, uv);

lazy_static! {
    static ref MODEL_CACHE: Mutex<HashMap<PathBuf, Weak<Model>>> = Mutex::new(HashMap::new());
}

/// A model.
#[derive(Clone, Debug)]
pub struct Model {
    /// The texture associated with the model, if any.
    pub texture: Option<()>,

    /// The vertices of the model.
    pub vertices: Vec<Vertex>,
}

impl Model {
    /// Fully loads an `.obj` to a Model.
    pub fn load_obj(path: impl AsRef<Path>) -> Fallible<Arc<Model>> {
        let mut cache = MODEL_CACHE.lock().unwrap();

        let path = canonicalize(path)?;
        if let Some(model) = cache.get(&path).and_then(Weak::upgrade) {
            debug!("Cache hit for {}!", path.display());
            return Ok(model);
        }

        let model = {
            let mut obj = Obj::load(&path)
                .with_context(|err| format_err!("When loading {}: {}", path.display(), err))?;
            if let Err(errs) = obj.load_mtls() {
                let mut msg = "Errors while loading MTLs:".to_string();
                for (file, err) in errs {
                    write!(msg, "\n{}: {}", file, err);
                }
                bail!("{}", msg);
            }
            Arc::new(Model::from(obj))
        };

        cache.insert(path, Arc::downgrade(&model));
        Ok(model)
    }

    /// Creates a model for a quad with the given vertices.
    pub fn quad(
        v1: (f32, f32, f32),
        v2: (f32, f32, f32),
        v3: (f32, f32, f32),
        v4: (f32, f32, f32),
        texture: Option<()>,
    ) -> Model {
        let v1 = Vector3::from(v1);
        let v2 = Vector3::from(v2);
        let v3 = Vector3::from(v3);
        let v4 = Vector3::from(v4);
        let u = v2 - v1;
        let v = v3 - v1;
        let normal = Vector3::new(
            u.y * v.z - u.z * v.y,
            u.z * v.x - u.x * v.z,
            u.x * v.y - u.y * v.x,
        );

        let v1 = Vertex::new(v1, normal, [0.0, 0.0]);
        let v2 = Vertex::new(v2, normal, [0.0, 1.0]);
        let v3 = Vertex::new(v3, normal, [1.0, 1.0]);
        let v4 = Vertex::new(v4, normal, [1.0, 0.0]);
        Model {
            texture,
            vertices: vec![v1, v2, v3, v3, v4, v1],
        }
    }
}

impl<'a> From<Obj<'a, SimplePolygon>> for Model {
    fn from(obj: Obj<'a, SimplePolygon>) -> Model {
        warn!("TODO Model::from Obj");
        Model {
            texture: None,
            vertices: Vec::new(),
        }
    }
}
