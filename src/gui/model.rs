use cgmath::{InnerSpace, Vector3};
use failure::{Fallible, ResultExt};
use glium::texture::RawImage2d;
use image;
use obj::{Material as MtlMaterial, Mtl, Obj, SimplePolygon};
use std::{
    collections::HashMap,
    fmt::Write,
    fs::{canonicalize, File},
    io::BufReader,
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
    static ref DEFAULT_MATERIAL: Arc<Material> = Arc::new(Material::flat([1.0, 0.0, 1.0]));
    static ref MATERIAL_CACHE: Mutex<HashMap<PathBuf, Weak<Material>>> = Mutex::new(HashMap::new());
    static ref MODEL_CACHE: Mutex<HashMap<PathBuf, Weak<Model>>> = Mutex::new(HashMap::new());
    static ref TEXTURE_CACHE: Mutex<HashMap<PathBuf, Weak<RawImage2d<'static, u8>>>> =
        Mutex::new(HashMap::new());
}

/// A model.
#[derive(Clone)]
pub struct Model {
    /// The material associated with the model.
    pub material: Arc<Material>,

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
        material: Option<Arc<Material>>,
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
        ).normalize();

        let v1 = Vertex::new(v1, normal, [0.0, 0.0]);
        let v2 = Vertex::new(v2, normal, [0.0, 1.0]);
        let v3 = Vertex::new(v3, normal, [1.0, 1.0]);
        let v4 = Vertex::new(v4, normal, [1.0, 0.0]);
        Model {
            material: material.unwrap_or_else(|| DEFAULT_MATERIAL.clone()),
            vertices: vec![v1, v2, v3, v3, v4, v1],
        }
    }

    /// Creates a model for a quad with the given vertices.
    pub fn quad_no_stretch(
        v1: (f32, f32, f32),
        v2: (f32, f32, f32),
        v3: (f32, f32, f32),
        v4: (f32, f32, f32),
        material: Option<Arc<Material>>,
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
        ).normalize();

        let h = (v2 - v1).magnitude();
        let w = (v4 - v1).magnitude();

        let v1 = Vertex::new(v1, normal, [0.0, 0.0]);
        let v2 = Vertex::new(v2, normal, [0.0, h]);
        let v3 = Vertex::new(v3, normal, [w, h]);
        let v4 = Vertex::new(v4, normal, [w, 0.0]);
        Model {
            material: material.unwrap_or_else(|| DEFAULT_MATERIAL.clone()),
            vertices: vec![v1, v2, v3, v3, v4, v1],
        }
    }
}

impl<'a> From<Obj<'a, SimplePolygon>> for Model {
    fn from(obj: Obj<'a, SimplePolygon>) -> Model {
        warn!("TODO Model::from Obj");
        Model {
            material: DEFAULT_MATERIAL.clone(),
            vertices: Vec::new(),
        }
    }
}

/// The material associated with the model.
pub struct Material {
    /// The ambient color.
    pub ambient: [f32; 4],

    /// The diffuse color.
    pub diffuse: [f32; 4],

    /// The texture, if any.
    pub texture: Option<Arc<RawImage2d<'static, u8>>>,
}

impl Material {
    /// Returns a material that is "flatly" of the given color. (i.e., no texture, no
    /// transparency).
    pub fn flat(color: impl Into<[f32; 3]>) -> Material {
        let color = color.into();
        let color = [color[0], color[1], color[2], 1.0];
        Material {
            ambient: color,
            diffuse: color,
            texture: None,
        }
    }

    /// Loads a material from a `.mtl` file.
    pub fn load_mtl(path: impl AsRef<Path>) -> Fallible<Arc<Material>> {
        let path = path.as_ref();
        let mut cache = MATERIAL_CACHE.lock().unwrap();

        let path = canonicalize(path)?;
        if let Some(material) = cache.get(&path).and_then(Weak::upgrade) {
            debug!("Cache hit for {}!", path.display());
            return Ok(material);
        }

        let mut file = File::open(&path).map(BufReader::new).with_context(|err| {
            format_err!("Couldn't open material file {}: {}", path.display(), err)
        })?;
        let mtl = Mtl::load(&mut file);
        let mtl: &MtlMaterial = match &mtl.materials as &[_] {
            &[] => bail!("No materials found in {}", path.display()),
            &[ref mtl] => mtl,
            _ => bail!("Too many materials found in {}", path.display()),
        };

        let color = |o: Option<[f32; 3]>| o.map(|[r, g, b]| [r, g, b, 1.0]).unwrap_or_default();

        let mtl = Arc::new(Material {
            ambient: color(mtl.ka),
            diffuse: color(mtl.kd),
            texture: match mtl.map_kd.as_ref() {
                Some(tex_path) => Some(load_texture(&path, tex_path)?),
                None => None,
            },
        });
        cache.insert(path, Arc::downgrade(&mtl));
        Ok(mtl)
    }
}

fn load_texture(
    base_path: impl AsRef<Path>,
    tex_path: impl AsRef<Path>,
) -> Fallible<Arc<RawImage2d<'static, u8>>> {
    let mut cache = TEXTURE_CACHE.lock().unwrap();

    let path = base_path
        .as_ref()
        .parent()
        .map(|p| p.join(tex_path.as_ref()))
        .unwrap_or_else(|| tex_path.as_ref().to_owned());
    let path = canonicalize(path)?;
    if let Some(texture) = cache.get(&path).and_then(Weak::upgrade) {
        debug!("Cache hit for {}!", path.display());
        return Ok(texture);
    }

    let img = image::open(&path)
        .with_context(|err| format_err!("Couldn't open image file {}: {}", path.display(), err))?
        .to_rgba();
    let dims = img.dimensions();
    let img = Arc::new(RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims));
    cache.insert(path, Arc::downgrade(&img));
    Ok(img)
}
