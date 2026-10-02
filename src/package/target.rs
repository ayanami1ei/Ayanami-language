use super::*;


/// What kind of artifact this package can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    Executable,
    StaticLib,
    DynamicLib,
}

impl TargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetType::Executable => "executable",
            TargetType::StaticLib => "static-lib",
            TargetType::DynamicLib => "dynamic-lib",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "executable" => Some(TargetType::Executable),
            "static-lib" => Some(TargetType::StaticLib),
            "dynamic-lib" => Some(TargetType::DynamicLib),
            _ => None,
        }
    }
}
