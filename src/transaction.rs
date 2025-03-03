use rust_decimal::Decimal;

use crate::ClientId;
use crate::input::{RawCsvTransaction, TransactionType};

pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

pub struct Deposit {
    pub client: ClientId,
    pub tx: u32,
    pub amount: Decimal,
}

pub struct Withdrawal {
    pub client: ClientId,
    pub tx: u32,
    pub amount: Decimal,
}

pub struct Dispute {
    pub client: ClientId,
    pub tx: u32,
}

pub struct Resolve {
    pub client: ClientId,
    pub tx: u32,
}

pub struct Chargeback {
    pub client: ClientId,
    pub tx: u32,
}

impl TryFrom<&RawCsvTransaction> for Transaction {
    type Error = String;

    /// Try to convert a `RawCsvTransaction` to a Transaction we can work with
    /// We try to express invariants that we cannot get through the raw data coming in
    /// For example, in the raw data a Deposit has an amount, but due to the format it is an `Option`
    /// Here we try to parse out that rough data into invariants like Deposits (which have an amount) and Disputes (which don't have an amount)
    fn try_from(value: &RawCsvTransaction) -> Result<Self, Self::Error> {
        let transaction = match value.r#type {
            TransactionType::Deposit => Transaction::Deposit(Deposit {
                client: value.client,
                tx: value.tx,
                amount: value
                    .amount
                    .ok_or(String::from("amount is mandatory when type is deposit"))?,
            }),
            TransactionType::Withdrawal => Transaction::Withdrawal(Withdrawal {
                client: value.client,
                tx: value.tx,
                amount: value
                    .amount
                    .ok_or(String::from("amount is mandatory when type is withdrawal"))?,
            }),

            TransactionType::Dispute => Transaction::Dispute(Dispute {
                client: value.client,
                tx: value.tx,
            }),

            TransactionType::Resolve => Transaction::Resolve(Resolve {
                client: value.client,
                tx: value.tx,
            }),
            TransactionType::Chargeback => Transaction::Chargeback(Chargeback {
                client: value.client,
                tx: value.tx,
            }),
        };

        Ok(transaction)
    }
}
