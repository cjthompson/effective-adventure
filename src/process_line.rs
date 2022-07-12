use anyhow::{anyhow, bail, Result};
use rust_decimal::{prelude::*, Decimal};

use crate::client::{
    Action, Clients, TransactionAction, TransactionRecord, TransactionState, TransactionUpdate,
    TransactionUpdateType,
};

pub trait ProcessLine {
    fn process_line(self, clients: &mut Clients) -> Result<()>;
}

impl ProcessLine for TransactionRecord {
    fn process_line(self, clients: &mut Clients) -> Result<()> {
        match self {
            TransactionRecord::Action(x) => x.process_line(clients),
            TransactionRecord::Update(x) => x.process_line(clients),
        }
    }
}

impl ProcessLine for TransactionAction {
    fn process_line(self, clients: &mut Clients) -> Result<()> {
        let t = self;
        let id = t.id;
        let client = clients.get_mut(t.client);
        // Ignore a transaction that has a duplicate key.
        if client.transactions.contains_key(&id) {
            bail!("Duplicate Transaction ID [{}]. Transaction Rejected.", id);
        }
        client.transactions.insert(id, t);
        // shadow t with a mutable reference to the item in the collection
        let t = client.transactions.get_mut(&id).unwrap();

        if client.locked {
            t.state = TransactionState::Rejected;
            bail!("Client account is locked. Transaction rejected.");
        }

        if t.amount < Decimal::zero() {
            bail!(
                "Invalid deposit amount [{}]. Deposits must be >= 0.",
                t.amount
            );
        }

        match t.r#type {
            Action::Deposit => {
                client.available = client.available.checked_add(t.amount).ok_or_else(|| {
                    t.state = TransactionState::Rejected;
                    anyhow!("Invalid amount. Deposit rejected.")
                })?
            }
            Action::Withdrawal => {
                if client.available < t.amount {
                    t.state = TransactionState::Rejected;
                    bail!("Insufficient funds. Withdrawal rejected.");
                }
                client.available = client.available.checked_sub(t.amount).ok_or_else(|| {
                    t.state = TransactionState::Rejected;
                    anyhow!("Invalid amount. Withdrawal rejected.")
                })?
            }
        }

        // We already bailed at all the places a transaction could be rejected
        t.state = TransactionState::Completed;

        Ok(())
    }
}

impl ProcessLine for TransactionUpdate {
    fn process_line(self, clients: &mut Clients) -> Result<()> {
        let t = self;
        let client = clients.get_mut(t.client);
        if client.locked {
            bail!("Client account is locked. All disputes and resolutions are rejected.");
        }

        let tx = client.transactions.get_mut(&t.tx_id).ok_or_else(|| {
            anyhow!("Account update action is invalid. No matching transaction exists.")
        })?;

        match t.r#type {
            TransactionUpdateType::Dispute => {
                if tx.state == TransactionState::Completed {
                    // use temporary variables to avoid having to revert the first change if the second fails
                    let held = client.held.checked_add(tx.amount).ok_or_else(|| {
                        tx.state = TransactionState::Rejected;
                        anyhow!("Invalid amount. Dispute rejected.")
                    })?;
                    // This could lead to a negative available balance
                    let available = client.available.checked_sub(tx.amount).ok_or_else(|| {
                        tx.state = TransactionState::Rejected;
                        anyhow!("Invalid amount. Dispute rejected.")
                    })?;
                    client.held = held;
                    client.available = available;
                    tx.state = TransactionState::Disputed;
                } else {
                    bail!(
                        "Dispute has no matching previous transaction. Dispute Rejected. {:#?}",
                        tx
                    );
                }
            }
            TransactionUpdateType::Resolve => {
                if tx.id == t.tx_id
                        && tx.state == TransactionState::Disputed
                        // Don't allow a resolution if insufficient funds are being held in dispute
                        && client.held >= tx.amount
                {
                    // use temporary variables to avoid having to revert the first change if the second fails
                    let held = client.held.checked_sub(tx.amount).ok_or_else(|| {
                        tx.state = TransactionState::Rejected;
                        anyhow!("Invalid amount. Resolve rejected.")
                    })?;
                    let available = client.available.checked_add(tx.amount).ok_or_else(|| {
                        tx.state = TransactionState::Rejected;
                        anyhow!("Invalid amount. Resolve rejected.")
                    })?;
                    client.held = held;
                    client.available = available;
                    tx.state = TransactionState::Resolved;
                } else {
                    bail!("Resolve has no valid matching previous transaction. Resolve Rejected.")
                }
            }
            // I assume that Chargeback is only for Withdrawals
            TransactionUpdateType::Chargeback => {
                if tx.id == t.tx_id
                    && tx.client == t.client
                    && tx.state == TransactionState::Disputed
                {
                    tx.state = TransactionState::Reversed;
                    client.locked = true;
                    // available funds doesn't change in this case, they just lose the held funds
                    client.held = client.held.checked_sub(tx.amount).ok_or_else(|| {
                        tx.state = TransactionState::Rejected;
                        anyhow!("Invalid amount. Resolve rejected.")
                    })?;
                } else {
                    bail!("Chargeback has no matching previous transaction. Chargeback Rejected.")
                }
            }
        }

        Ok(())
    }
}
