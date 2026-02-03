pub mod implement;

#[derive(Debug, Clone)]
enum ErrorType{
    Error,
    Warning
}

#[derive(Debug)]
pub struct Error{
    _type:ErrorType,
    inner:anyhow::Error
}