use std::io::Error;

pub fn return_when_error<T>(result: std::io::Result<T>, error_message: &str) -> std::io::Result<T> {
    match result {
        Ok(_) => result,
        Err(e) => Err(Error::new(e.kind(), error_message)),
    }
}
