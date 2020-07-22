use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Keyword(String);

impl Keyword {
    pub fn new(val: String) -> Self {
        Self(val)
    }

    pub fn from_str(val: &str) -> Self {
        Self(String::from(val))
    }

    pub fn string_value(&self) -> &'_ String {
        &self.0
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.string_value())
    }
}
