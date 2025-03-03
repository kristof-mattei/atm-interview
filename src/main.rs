mod input;
mod ledger;
mod transaction;

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use color_eyre::eyre::{self};
use input::parse_csv;
use ledger::Ledger;
use rust_decimal::Decimal;

type ClientId = u16;
type TransactionId = u32;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(required = true)]
    path_to_file: PathBuf,
}

async fn run_tasks(cli: Cli) -> Result<(), eyre::Report> {
    let cli = Arc::new(cli);

    // our code reads csv, but we built it with the idea that it can read from an endless stream of messages, passing them to a
    // ledger, which processes them, and once done, outputs its state

    // arbitrary transaction channel to facilitate easy concurreny
    let (sender, mut receiver) = tokio::sync::mpsc::channel(10);

    let tasks = tokio_util::task::TaskTracker::new();

    // parse csv task
    {
        tasks.spawn(async move { parse_csv(&cli.path_to_file, sender).await });
    }

    // ledger task
    {
        tasks.spawn(async move {
            let mut ledger = Ledger::new();

            while let Some(transaction) = receiver.recv().await {
                ledger.process(&transaction);
            }

            let mut lock = std::io::stdout().lock();

            writeln!(lock, "client,available,held,total,locked").expect("Failed to write output");

            for (client_id, account) in ledger.accounts {
                let account_balance = account.balance.normalize();
                let disputes_amount = account
                    .disputes
                    .iter()
                    .map(|tx| {
                        account
                            .deposits
                            .get(tx)
                            .expect("Couldn't find disputed deposit")
                    })
                    .sum::<Decimal>()
                    .normalize();
                let total_amount = (account.balance + disputes_amount).normalize();

                writeln!(
                    lock,
                    "{},{},{},{},{}",
                    client_id, account_balance, disputes_amount, total_amount, account.locked
                )
                .expect("Failed to write output");
            }

            // lock is released
        });
    }

    tasks.close();

    tasks.wait().await;

    Ok(())
}

fn main() -> Result<(), eyre::Report> {
    let cli = match Cli::try_parse() {
        Ok(cli_input) => cli_input,
        Err(error) => {
            error.exit();
        },
    };

    // initialize the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // start service
    let result: Result<(), color_eyre::Report> = rt.block_on(run_tasks(cli));

    result
}
