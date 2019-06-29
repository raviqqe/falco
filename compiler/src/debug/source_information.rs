use super::location::Location;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceInformation {
    filename: String,
    location: Location,
    line: String,
}

impl SourceInformation {
    pub fn new(filename: String, location: Location, line: String) -> Self {
        Self {
            filename,
            location,
            line,
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn line(&self) -> &str {
        &self.line
    }
}

impl std::fmt::Display for SourceInformation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            formatter,
            "{}:{}:{}:{}",
            self.filename,
            self.location.line_number(),
            self.location.column_number(),
            self.line,
        )
    }
}
