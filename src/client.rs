use std::collections::HashMap;

use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum TransactionRecord {
    Action(TransactionAction),
    Update(TransactionUpdate),
}

#[derive(Debug, Clone)]
pub struct TransactionAction {
    pub id: u32,
    pub client: u16,
    pub r#type: Action,
    pub amount: Decimal,
    pub state: TransactionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionUpdateType {
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone)]
pub struct TransactionUpdate {
    pub client: u16,
    pub tx_id: u32,
    pub r#type: TransactionUpdateType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Pending,
    Completed,
    Disputed,
    Resolved,
    Reversed,
    Rejected,
}

#[derive(Debug, Clone, Default)]
pub struct Clients {
    clients: HashMap<u16, Client>,
}

impl Clients {
    pub fn get_mut(&mut self, id: u16) -> &mut Client {
        if !self.clients.contains_key(&id) {
            self.clients.insert(id, Client::new(id));
        }
        self.clients.get_mut(&id).unwrap() // Won't panic because the record was just inserted
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    // Don't let people change this
    id: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
    pub transactions: HashMap<u32, TransactionAction>,
}

impl Client {
    fn new(id: u16) -> Self {
        Self {
            id,
            available: Decimal::zero(),
            held: Decimal::zero(),
            locked: false,
            transactions: HashMap::new(),
        }
    }

    pub fn total(&self) -> Decimal {
        self.available + self.held
    }

    pub fn id(&self) -> u16 {
        self.id
    }
}

impl IntoIterator for Clients {
    type Item = <HashMap<u16, Client> as IntoIterator>::Item;
    type IntoIter = <HashMap<u16, Client> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.clients.into_iter()
    }
}
