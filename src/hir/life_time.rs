pub(crate) enum LifeTimeMode{
    Create,
    Use(UseMode),
    Escape(EscapeMode),
    Delete,
}

pub(crate) enum EscapeMode{
    InCall,
    InReturn
}

pub(crate) enum UseMode{
    InExpr,
}