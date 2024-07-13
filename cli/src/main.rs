use sqlx::SqlitePool;
use vending_machine::application::VendingMachine;

use crate::contracts::PromptPerspective;
use crate::terminals::CliTerminal;

mod contracts;
mod terminals;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let product_repository = Box::new(InMemoryProductRepository::default());
    // let sale_repository = Box::new(infrastructure::in_memory::InMemorySaleRepository::default());
    dotenvy::dotenv().ok();

    let pool = SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;
    let product_repository = Box::new(infrastructure::sqlite::SqliteProductRepository::new(
        pool.clone(),
    ));
    let sale_repository = Box::new(infrastructure::sqlite::SqliteSaleRepository::new(pool));
    let payment_terminal = Box::new(terminals::CliPaymentTerminal);

    let vending_machine =
        VendingMachine::new(product_repository, sale_repository, payment_terminal);
    let mut terminal = PromptPerspective::GuestUnlocked(CliTerminal::new(vending_machine));

    loop {
        terminal = terminal.dispatch().await;
    }
}
