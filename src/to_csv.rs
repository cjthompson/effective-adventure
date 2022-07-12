use csv::Writer;
use rust_decimal::Decimal;
use serde::Serialize;
use std::io::{stdout, Result};

use crate::{Client, Clients};

#[derive(Debug, Clone, Serialize)]
struct SerializedClient {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl From<Client> for SerializedClient {
    fn from(x: Client) -> Self {
        Self {
            client: x.id(),
            available: x.available.round_dp(4),
            held: x.held.round_dp(4),
            total: x.total().round_dp(4),
            locked: x.locked,
        }
    }
}

impl Clients {
    pub fn to_csv(self) -> Result<()> {
        let mut wtr = Writer::from_writer(stdout());
        for client in self
            .into_iter()
            .map(|(_, value)| Into::<SerializedClient>::into(value))
        {
            wtr.serialize(client)?;
        }

        wtr.flush()
    }
}
