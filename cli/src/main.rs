use infrastructure::sqlite::{DbConn, SqliteProductRepository, SqliteSaleRepository};
use vending_machine::application::states::{Guest, Unlocked};
use vending_machine::application::VendingMachine;
use yadir::core::primitives::{DIManager, Lifetime};

use crate::contracts::PromptPerspective;
use crate::terminals::{CliPaymentTerminal, CliTerminal};

mod contracts;
mod terminals;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let mut manager = DIManager::default();

    manager
        .register::<DbConn>(Some(Lifetime::Singleton))
        .await
        .register::<SqliteProductRepository>(Some(Lifetime::Singleton))
        .await
        .register::<SqliteSaleRepository>(Some(Lifetime::Singleton))
        .await
        .register::<CliPaymentTerminal>(Some(Lifetime::Singleton))
        .await
        .register::<VendingMachine<Guest, Unlocked>>(Some(Lifetime::Singleton))
        .await
        .register::<CliTerminal<Guest, Unlocked>>(Some(Lifetime::Singleton))
        .await;

    let terminal = manager
        .resolve::<CliTerminal<Guest, Unlocked>>()
        .await
        .unwrap()
        .extract();

    let mut terminal = PromptPerspective::GuestUnlocked(terminal);

    loop {
        terminal = terminal.dispatch().await;
    }
}
