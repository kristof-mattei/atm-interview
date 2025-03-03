use std::path::Path;

use color_eyre::eyre::{self, Context};
use csv::{ReaderBuilder, Trim};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::ClientId;
use crate::transaction::Transaction;

#[derive(Deserialize, Debug)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(Deserialize, Debug)]
pub struct RawCsvTransaction {
    #[serde(rename = "type")]
    pub r#type: TransactionType,
    pub client: ClientId,
    pub tx: u32,
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    pub amount: Option<Decimal>,
}

pub async fn parse_csv(
    path_to_file: &Path,
    sender: tokio::sync::mpsc::Sender<Transaction>,
) -> Result<(), eyre::Error> {
    // try and read the file, using exists to detect existance can return false positive if the file is deleted in between checking and reading
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(path_to_file)
        .wrap_err("Unable to read file, does it exist?")?;

    for transaction in reader.deserialize::<RawCsvTransaction>() {
        let transaction = transaction?;

        sender
            .send(
                (&transaction)
                    .try_into()
                    .map_err(|error| eyre::Report::msg(error))?,
            )
            .await?;
    }

    Ok(())
}
