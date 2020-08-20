use crate::{Exception, ExceptionValue as EV};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone, Debug)]
pub struct Locker<T> {
    val: Arc<RwLock<T>>,
}

impl<T> Locker<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: Arc::new(RwLock::new(val)),
        }
    }

    pub fn read(&self) -> Result<RwLockReadGuard<T>, Exception> {
        match self.val.read() {
            Ok(val) => Ok(val),
            Err(_) => Err(Exception::new(
                EV::Concurrency,
                None,
                Some(
                    "could not safely find value in memory (has concurrency gone wrong?)"
                        .to_string(),
                ),
            )),
        }
    }

    pub fn write(&self) -> Result<RwLockWriteGuard<T>, Exception> {
        match self.val.write() {
            Ok(val) => Ok(val),
            Err(_) => Err(Exception::new(
                EV::Concurrency,
                None,
                Some(
                    "could not safely find value in memory (has concurrency gone wrong?)"
                        .to_string(),
                ),
            )),
        }
    }
}
