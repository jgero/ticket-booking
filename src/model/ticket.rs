use serde::Serialize;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone)]
pub enum Status {
    Unchecked,
    Available,
    Ordered(Uuid),
    Sold,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unchecked => write!(f, "unchecked"),
            Self::Available => write!(f, "available"),
            Self::Ordered(uuid) => write!(f, "ordered[by={}]", uuid),
            Self::Sold => write!(f, "sold"),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Ticket {
    visible_id: String,
    #[serde(skip_serializing)]
    status: Status,
}

impl From<String> for Ticket {
    fn from(value: String) -> Self {
        Ticket {
            visible_id: value,
            status: Status::Unchecked,
        }
    }
}

impl Display for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ticket[id={},status={}]", self.visible_id, self.status)
    }
}
