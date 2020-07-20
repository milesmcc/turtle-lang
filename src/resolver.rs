use crate::{exp, parse, CallSnapshot, Exception, ExceptionValue as EV, Expression};
use relative_path::RelativePath;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub fn resolve_resource<'a>(
    path: &str,
    snapshot: Arc<RwLock<CallSnapshot<'a>>>,
    via: &Expression<'a>,
) -> Result<Expression<'a>, Exception<'a>> {
    let source_path_opt = match via.source() {
        Some(source) => match source.location() {
            Some(location) => Some(location),
            None => None,
        },
        None => None,
    };

    let working_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => exp!(
            EV::InvalidIncludePath(String::from(path)),
            snapshot,
            "could not establish working directory (the environment is unknown)".to_string()
        ),
    };

    let relative_dir = match source_path_opt {
        Some(source_path) => match fs::metadata(&source_path) {
            Ok(metadata) => match metadata.is_dir() {
                true => PathBuf::from(source_path),
                false => match PathBuf::from(source_path).parent() {
                    Some(parent) => PathBuf::from(parent),
                    None => working_dir,
                },
            },
            Err(_) => working_dir,
        },
        None => working_dir,
    };

    let relative_dir_composed = match RelativePath::from_path(&path) {
        Ok(relative) => relative,
        Err(err) => exp!(
            EV::InvalidIncludePath(String::from(path)),
            snapshot,
            format!(
                "could not understand include path ({}; all includes must be relative)",
                err
            )
        ),
    };

    let content = match fs::read_to_string(&relative_dir_composed.to_path(relative_dir)) {
        Ok(value) => value,
        Err(val) => exp!(
            EV::InvalidIncludePath(path.to_string()),
            snapshot,
            format!("unable to read file ({})", val)
        ),
    };

    let parsed = parse(
        &content,
        &path.to_string(),
        via.clone_env(),
    )?;

    let mut return_val = Expression::nil();
    for mut exp in parsed {
        return_val = exp.eval(CallSnapshot::new(&exp, &snapshot))?;
    }
    Ok(return_val)
}