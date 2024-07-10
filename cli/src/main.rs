mod terminals;

use infrastructure::in_memory::InMemoryProductRepository;
use vending_machine::application::auth::AuthResult;
use vending_machine::application::VendingMachine;
use vending_machine::domain::entities::{Name, Password, Price, Product, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let product_repository = Box::new(InMemoryProductRepository::new());
    let payment_terminal = Box::new(terminals::CliPaymentTerminal);

    let vending_machine = VendingMachine::new(product_repository, payment_terminal);

    let (username, password) = ("supplier", "supplier_pass");
    let (username, password) = (Name::parse(username)?, Password::parse(password)?);

    println!("{:?}", vending_machine.look_up());

    match vending_machine.login(&username, &password) {
        AuthResult::SuccessAdmin(machine) => {
            println!("{:?}", machine.look_up());
        }
        AuthResult::SuccessSupplier(mut machine) => {
            let product = Product {
                column_id: Value::parse("1")?,
                name: Name::parse("Coca Cola")?,
                price: Price::parse("1.5")?,
                quantity: Value::parse("10")?,
            };
            machine.supply_new_product(product)?;
            println!("{:?}", machine.look_up());

            let mut machine = machine.logout();
            machine.buy(Value::parse("1")?, Value::parse("1")?)?;

            println!("{:?}", machine.look_up());
        }
        AuthResult::Failure(machine) => {
            println!("Login failed");
            println!("{:?}", machine.look_up());
        }
    }

    Ok(())
}
