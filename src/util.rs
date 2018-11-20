//! Miscellaneous utilities.

use failure::{Error, Fallible, ResultExt};
use obj::{Obj, SimplePolygon};
use std::{fs::File, io::Read, path::Path, str::FromStr, sync::Arc};

/// Quick `impl typemap::Key<Value = Self>`.
macro_rules! impl_Component {
    ($($ty:ty),*) => {
        $(impl $crate::typemap::Key for $ty {
            type Value = $ty;
        })*
    }
}

/// Fully loads an `.obj`.
pub fn load_obj(path: impl AsRef<Path>) -> Fallible<Arc<Obj<'static, SimplePolygon>>> {
    // TODO: Caching

    let mut obj = Obj::<SimplePolygon>::load(path.as_ref())
        .with_context(|_| format_err!("When loading {}", path.as_ref().display()))?;
    if let Err(errs) = obj.load_mtls() {
        let mut msg = "Errors while loading MTLs:".to_string();
        for (file, err) in errs {
            msg.push('\n');
            msg += &file;
            msg += ": ";
            msg += &err.to_string();
        }
        bail!("{}", msg);
    }
    println!(
        "{:?}",
        obj.objects.into_iter().map(|o| o.name).collect::<Vec<_>>()
    );
    //assert!(obj.objects.is_empty());

    Ok(Arc::new(Obj {
        position: obj.position,
        texture: obj.texture,
        normal: obj.normal,
        objects: Vec::new(),
        material_libs: obj.material_libs,
        path: obj.path,
    }))
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
pub fn read_file_and_parse_to<E, P, T>(path: P) -> Fallible<T>
where
    E: Into<Error>,
    P: AsRef<Path>,
    T: FromStr<Err = E>,
{
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    drop(file);

    match buf.parse() {
        Ok(data) => Ok(data),
        Err(err) => Err(err.into()),
    }
}
