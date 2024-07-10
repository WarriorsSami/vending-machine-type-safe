use infrastructure::in_memory::InMemoryProductRepository;
use vending_machine::application::auth::{AuthResult, Guest};
use vending_machine::application::VendingMachine;
use vending_machine::domain::entities::{Name, Password, Price, Product};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let product_repository = Box::new(InMemoryProductRepository::new());
    let vending_machine = VendingMachine::<Guest>::new(product_repository);

    let (username, password) = ("admin", "admin_pass");
    let (username, password) = (Name::parse(username)?, Password::parse(password)?);

    println!("{:?}", vending_machine.look_up());

    match vending_machine.login(&username, &password) {
        AuthResult::Success(mut vending_machine) => {
            let product = Product {
                column_id: 1,
                name: Name::parse("Coca Cola")?,
                price: Price::parse("1.5")?,
                quantity: 10,
            };
            vending_machine.supply_new_product(product)?;

            println!("{:?}", vending_machine.look_up());

            let vending_machine = vending_machine.logout();

            println!("{:?}", vending_machine.look_up());
        },
        AuthResult::Failure(vending_machine) => {
            println!("Login failed");
            println!("{:?}", vending_machine.look_up());
        },
    }

    Ok(())
}
