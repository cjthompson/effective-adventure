use anyhow::Result;
use std::path::Path;

use rust_decimal::{prelude::*, Decimal};
use serde::{Deserialize, Serialize};

use crate::client::{
    Action, TransactionAction, TransactionRecord, TransactionState, TransactionUpdate,
    TransactionUpdateType,
};

#[derive(Debug, Deserialize)]
pub struct InputTransaction {
    r#type: InputTransactionType,
    client: u16,
    tx: u32,
    #[serde(default)]
    amount: Option<Decimal>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InputTransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub(crate) fn parse<P: AsRef<Path>>(
    filename: P,
) -> Result<impl Iterator<Item = TransactionRecord>> {
    let reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(filename)?;

    Ok(reader
        .into_deserialize()
        .filter_map(|result| {
            result.map_or_else(
                |err| {
                    // The requirements document doesn't specify what to do when there is invalid data
                    eprintln!("Skipping invalid input: {}", err);
                    None
                },
                |x| Some(x),
            )
        })
        .map(|input_tx: InputTransaction| {
            match input_tx.r#type {
                InputTransactionType::Deposit => {
                    TransactionRecord::Action(TransactionAction {
                        id: input_tx.tx,
                        client: input_tx.client,
                        // Do not allow deposits with negative numbers
                        amount: input_tx.amount.unwrap_or_default().max(Decimal::zero()),
                        r#type: Action::Deposit,
                        state: TransactionState::Pending,
                    })
                }
                InputTransactionType::Withdrawal => {
                    TransactionRecord::Action(TransactionAction {
                        id: input_tx.tx,
                        client: input_tx.client,
                        // Do not allow withdrawals with negative numbers
                        amount: input_tx.amount.unwrap_or_default().max(Decimal::zero()),
                        r#type: Action::Withdrawal,
                        state: TransactionState::Pending,
                    })
                }
                InputTransactionType::Dispute => TransactionRecord::Update(TransactionUpdate {
                    client: input_tx.client,
                    tx_id: input_tx.tx,
                    r#type: TransactionUpdateType::Dispute,
                }),
                InputTransactionType::Resolve => TransactionRecord::Update(TransactionUpdate {
                    client: input_tx.client,
                    tx_id: input_tx.tx,
                    r#type: TransactionUpdateType::Resolve,
                }),
                InputTransactionType::Chargeback => TransactionRecord::Update(TransactionUpdate {
                    client: input_tx.client,
                    tx_id: input_tx.tx,
                    r#type: TransactionUpdateType::Chargeback,
                }),
            }
        }))
}
