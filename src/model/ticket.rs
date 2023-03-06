use serde::{Serialize, Deserialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone)]
pub struct Ticket {
    pub visible_id: String,
}

impl From<String> for Ticket {
    fn from(value: String) -> Self {
        Ticket {
            visible_id: value,
        }
    }
}

impl Display for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ticket[id={}]", self.visible_id)
    }
}
