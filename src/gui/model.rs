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
}

implement_vertex!(Vertex, xyz);

lazy_static! {
    static ref MODEL_CACHE: Mutex<HashMap<PathBuf, Weak<Model>>> = Mutex::new(HashMap::new());
}

/// A model.
#[derive(Clone, Copy, Debug)]
pub struct Model {
    // TODO
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
                .with_context(|_| format_err!("When loading {}", path.display()))?;
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
}

impl<'a> From<Obj<'a, SimplePolygon>> for Model {
    fn from(obj: Obj<'a, SimplePolygon>) -> Model {
        warn!("TODO Model::from");
        Model {}
    }
}
