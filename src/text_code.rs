use std::fmt::Display;

pub trait TextCode {
    fn as_primary_header(&self) -> String
    where
        Self: Display,
    {
        format!("\x1b[1;32m{self}\x1b[0m")
    }

    fn as_secondary_header(&self) -> String
    where
        Self: Display,
    {
        format!("\x1b[1;34m{self}\x1b[0m")
    }

    fn as_error(&self) -> String
    where
        Self: Display,
    {
        format!("\x1b[1;31m{self}\x1b[0m")
    }
}

impl TextCode for String {}
impl TextCode for &str {}
