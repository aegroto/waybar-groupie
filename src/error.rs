pub enum Error {
    DataFetch(&'static str),
    WindowDataParsing(&'static str),
    ShellCommand(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DataFetch(msg) => write!(f, "Data fetching error: {}", msg),
            Error::WindowDataParsing(msg) => write!(f, "Wiindow data parsing error: {}", msg),
            Error::ShellCommand(msg) => write!(f, "Shell command error: {}", msg),
        }
    }
}

impl Error {
    pub fn as_string(&self) -> String {
        format!("{}", self)
    }
}
