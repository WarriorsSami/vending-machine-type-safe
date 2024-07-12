use infrastructure::in_memory::InMemoryProductRepository;
use vending_machine::application::VendingMachine;

use crate::contracts::PromptPerspective;
use crate::terminals::CliTerminal;

mod contracts;
mod terminals;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let product_repository = Box::new(InMemoryProductRepository::default());
    let sale_repository = Box::new(infrastructure::in_memory::InMemorySaleRepository::default());
    let payment_terminal = Box::new(terminals::CliPaymentTerminal);

    let vending_machine =
        VendingMachine::new(product_repository, sale_repository, payment_terminal);
    let mut terminal = PromptPerspective::GuestUnlocked(CliTerminal::new(vending_machine));

    loop {
        terminal = terminal.dispatch()?;
    }
}
