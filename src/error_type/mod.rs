pub(crate) mod implement;

#[derive(Debug, Clone)]
enum ErrorType{
    Error,
    Warning
}

#[derive(Debug)]
pub(crate) struct Error{
    _type:ErrorType,
    inner:anyhow::Error
}