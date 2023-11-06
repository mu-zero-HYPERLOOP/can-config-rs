
pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    InvalidRange(String),
    InvalidType(String),
    DuplicatedSignal(String),
    DuplicatedEnumEntry(String),
    DuplicatedStructAttribute(String),
    UndefinedType(String),
    InvalidDecimalDefinition(String),
    FailedToResolveId,
}

