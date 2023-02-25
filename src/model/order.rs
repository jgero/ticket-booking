use std::fmt::Display;

use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ticket::Ticket;

#[derive(Serialize, Clone)]
pub struct Order {
    uuid: Uuid,
    pub issuer: String,
    tickets: Vec<Ticket>,
}

impl Order {
    pub fn new(new_order: NewOrder, issuer: String) -> Self {
        let uuid = Uuid::new_v4();
        debug!("new order with uuid {} for issuer {}", uuid, issuer);
        Order {
            uuid,
            issuer,
            tickets: new_order
                .ticket_visible_ids
                .iter()
                .map(|v| Ticket::from(v.to_owned()))
                .collect(),
        }
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Order[uuid={},issuer={},tickets={}]",
            self.uuid,
            self.issuer,
            self.tickets
                .iter()
                .map(Ticket::to_string)
                .collect::<String>()
        )
    }
}

#[derive(Deserialize)]
pub struct NewOrder {
    ticket_visible_ids: Vec<String>,
}
