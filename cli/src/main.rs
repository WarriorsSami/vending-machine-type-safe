use yadir::core::DIManager;

use infrastructure::sqlite::{SqliteProductRepository, SqliteSaleRepository};
use vending_machine::application::states::{Guest, Unlocked};
use vending_machine::application::VendingMachine;

use crate::contracts::PromptPerspective;
use crate::terminals::{CliPaymentTerminal, CliTerminal};

mod contracts;
mod terminals;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let mut manager = DIManager::default();

    // manager.build::<InMemoryProductRepository>().await;
    // manager.build::<InMemorySaleRepository>().await;

    manager.build::<SqliteProductRepository>().await;
    manager.build::<SqliteSaleRepository>().await;
    manager.build::<CliPaymentTerminal>().await;
    manager.build::<VendingMachine<Guest, Unlocked>>().await;
    let terminal = manager
        .build::<CliTerminal<Guest, Unlocked>>()
        .await
        .unwrap()
        .lock()
        .unwrap()
        .clone();

    let mut terminal = PromptPerspective::GuestUnlocked(terminal);

    loop {
        terminal = terminal.dispatch().await;
    }
}
