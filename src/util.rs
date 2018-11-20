//! Miscellaneous utilities.

use failure::{Error, Fail, Fallible, ResultExt};
use std::{fs::File, io::Read, path::Path, str::FromStr};

/// Quick `impl typemap::Key<Value = Self>`.
macro_rules! impl_Component {
    ($($ty:ty),*) => {
        $(impl $crate::typemap::Key for $ty {
            type Value = $ty;
        })*
    }
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
        .with_context(|_| format_err!("Couldn't open {}", path.as_ref().display()))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .with_context(|_| format_err!("Couldn't read from {}", path.as_ref().display()))?;
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
        Err(err) => Err(err
            .into()
            .context(format_err!("Couldn't parse {}", path.as_ref().display()))
            .into()),
    }
}
