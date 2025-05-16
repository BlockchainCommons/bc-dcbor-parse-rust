use logos::Span;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Error(String, Span);
impl Error {
    pub fn new(msg: impl Into<String>, span: Span) -> Self {
        Self(msg.into(), span)
    }

    pub fn message(&self) -> &str {
        &self.0
    }

    pub fn span(&self) -> &Span {
        &self.1
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for anyhow::Error {
    fn from(err: Error) -> Self {
        anyhow::anyhow!("{:?}: {}", err.span(), err.message())
    }
}
