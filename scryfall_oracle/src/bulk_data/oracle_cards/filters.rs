use std::fmt;
#[derive(Debug)]
pub enum OracleFilter {
    Unsets,
    Modern,
    Premodern,
    UnknownEvent,
}
#[derive(Debug)]
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
        };

        write!(f, "{value}")
    }
}
