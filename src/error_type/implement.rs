use crate::error_type::Error;
use core::fmt;
use owo_colors::OwoColorize;

impl Error {
    pub(crate) fn new_error(s: String) -> Error {
        Error {
            _type: super::ErrorType::Error,
            inner: anyhow::Error::msg(s),
        }
    }

    #[allow(unused)]
    pub(crate) fn new_warning(s: String) -> Error {
        Error {
            _type: super::ErrorType::Warning,
            inner: anyhow::Error::msg(s),
        }
    }

    pub(crate) fn with_context_front(&mut self, s:String) -> Error {
        let mut _s=s;
        _s.push_str(&self.inner.to_string());
        self.inner = anyhow::Error::msg(_s.clone());
        self.clone()
    }

    #[allow(unused)]
    pub(crate) fn with_context_back(&mut self, s:String) -> Error {
        let mut _s = self.inner.to_string();
        _s.push_str(&s);

        self.inner = anyhow::Error::msg(_s);
        self.clone()
    }

    #[allow(unused)]
    pub(crate) fn msg(&self) -> String {
        self.inner.to_string()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self._type {
            super::ErrorType::Error => {
                write!(f, "{}", "Error: ".red())?;
            }
            super::ErrorType::Warning => {
                write!(f, "{}", "Warning: ".yellow())?;
            }
        };
        write!(f, "{}", self.inner.to_string())
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Self {
            _type: self._type.clone(),
            inner: anyhow::Error::msg(self.inner.to_string()),
        }
    }
}
