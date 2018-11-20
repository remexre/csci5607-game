//! Miscellaneous utilities.

use failure::{Error, Fallible};
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
