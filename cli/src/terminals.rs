use std::error::Error;
use vending_machine::domain::entities::Price;
use vending_machine::domain::interfaces::{PaymentTerminal, Terminal};

pub struct CliPaymentTerminal;

impl Terminal for CliPaymentTerminal {
    fn prompt(&self, message: &str) -> Result<(), Box<dyn Error>> {
        println!("{}", message);
        Ok(())
    }
}

impl PaymentTerminal for CliPaymentTerminal {
    fn request(&self) -> Result<Price, Box<dyn Error>> {
        let mut amount = String::new();
        println!("Please insert the amount: ");
        std::io::stdin().read_line(&mut amount)?;

        Price::parse(amount.trim())
    }

    fn refund(&self, amount: Price) -> Result<(), Box<dyn Error>> {
        println!("Refunding: {:?}", amount);
        Ok(())
    }
}
