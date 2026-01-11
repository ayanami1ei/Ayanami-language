pub(crate) mod implement;

pub (crate) enum Token{
    Identifier(String),
    Operator(String),
    Keyword(String),
    Num(f64),
}