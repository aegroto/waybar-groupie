pub enum Error {
    DataFetchError(&'static str),
    WindowDataParsingError(&'static str),
    ShellCommandError(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DataFetchError(msg) => write!(f, "Data fetching error: {}", msg),
            Error::WindowDataParsingError(msg) => write!(f, "Wiindow data parsing error: {}", msg),
            Error::ShellCommandError(msg) => write!(f, "Shell command error: {}", msg),
        }
    }
}

impl Error {
    pub fn as_string(&self) -> String {
        format!("{}", self)
    }
}
