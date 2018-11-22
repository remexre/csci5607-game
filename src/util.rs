//! Miscellaneous utilities.

use failure::{Error, Fallible, ResultExt};
use glium::texture::RawImage2d;
use image;
use serde::Deserialize;
use serde_json::from_reader;
use std::{
    collections::HashMap,
    fs::{canonicalize, File},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex, Weak},
};

/// Quick `impl typemap::Key<Value = Self>`.
macro_rules! impl_Component {
    ($($ty:ty),*) => {
        $(impl $crate::typemap::Key for $ty {
            type Value = $ty;
        })*
    }
}

/// Loads a texture.
pub fn load_texture(
    base_path: impl AsRef<Path>,
    tex_path: impl AsRef<Path>,
) -> Fallible<Arc<RawImage2d<'static, u8>>> {
    lazy_static! {
        static ref TEXTURE_CACHE: Mutex<HashMap<PathBuf, Weak<RawImage2d<'static, u8>>>> =
            Mutex::new(HashMap::new());
    }

    let mut cache = TEXTURE_CACHE.lock().unwrap();

    let path = base_path
        .as_ref()
        .parent()
        .map(|p| p.join(tex_path.as_ref()))
        .unwrap_or_else(|| tex_path.as_ref().to_owned());
    let path = canonicalize(&path)
        .with_context(|err| format_err!("While canonicalizing {}: {}", path.display(), err))?;
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

/// Logs an error, including its causes and backtrace (if possible).
pub fn log_err(err: Error) {
    let mut first = true;
    let num_errs = err.iter_chain().count();
    if num_errs <= 1 {
        error!("{}", err);
    } else {
        for cause in err.iter_chain() {
            if first {
                first = false;
                error!("           {}", cause);
            } else {
                error!("caused by: {}", cause);
            }
        }
    }
    let bt = err.backtrace().to_string();
    if bt != "" {
        error!("{}", bt);
    }
}

/// Reads a file and parses it.
pub fn read_file(path: impl AsRef<Path>) -> Fallible<String> {
    let mut file = File::open(path.as_ref())
        .with_context(|err| format_err!("Couldn't open {}: {}", path.as_ref().display(), err))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).with_context(|err| {
        format_err!("Couldn't read from {}: {}", path.as_ref().display(), err)
    })?;
    drop(file);
    Ok(buf)
}

/// Reads a file and parses it.
pub fn read_file_and_parse_to<E, P, T>(path: P) -> Fallible<T>
where
    E: Into<Error>,
    P: AsRef<Path>,
    T: FromStr<Err = E>,
{
    match read_file(path.as_ref()).map_err(Error::from)?.parse() {
        Ok(data) => Ok(data),
        Err(err) => {
            let err = err.into();
            let ctx_err = format_err!("Couldn't parse {}: {}", path.as_ref().display(), err);
            Err(err.context(ctx_err).into())
        }
    }
}

/// Reads a file and parses it as JSON.
pub fn read_file_and_unjson<P, T>(path: P) -> Fallible<T>
where
    P: AsRef<Path>,
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(path.as_ref())
        .with_context(|err| format_err!("Couldn't open {}: {}", path.as_ref().display(), err))?;
    let data = from_reader(file).with_context(|err| {
        format_err!(
            "Couldn't parse {} as JSON: {}",
            path.as_ref().display(),
            err
        )
    })?;
    Ok(data)
}
