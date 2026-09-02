use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum OracleFilter {
    Unsets,
    Modern,
    Premodern,
    UnknownEvent,
    EverythingElse,
    // TODO Add a Planeswalker filter
}

#[derive(Debug, PartialEq, Eq)]
pub struct OracleFilters {
    pub filters: Vec<OracleFilter>,
}

impl OracleFilters {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn from_vec(filters: Vec<OracleFilter>) -> Self {
        Self { filters }
    }
}

impl fmt::Display for OracleFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            OracleFilter::Unsets => "Unsets",
            OracleFilter::Modern => "Modern",
            OracleFilter::Premodern => "Premodern",
            OracleFilter::UnknownEvent => "Unknown Event",
            OracleFilter::EverythingElse => "Everything Else",
        };

        write!(f, "{value}")
    }
}
