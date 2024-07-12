use crate::contracts::{
    AdminLockedCommand, AdminUnlockedCommand, GuestLockedCommand, GuestUnlockedCommand,
    PromptPerspective, SupplierLockedCommand, SupplierUnlockedCommand,
};
use std::error::Error;
use vending_machine::application::states::{
    Admin, AuthResult, Authenticated, Guest, LockStatus, Locked, Role, Supplier, Unlocked,
};
use vending_machine::application::VendingMachine;
use vending_machine::domain::entities::{Name, Password, Price, Product, Value};
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
        println!("Here's your refund: {}", amount.as_value());
        Ok(())
    }
}

pub struct CliTerminal<U: Role, L: LockStatus> {
    vending_machine: VendingMachine<U, L>,
}

impl CliTerminal<Guest, Unlocked> {
    pub fn new(vending_machine: VendingMachine<Guest, Unlocked>) -> Self {
        Self { vending_machine }
    }
}

impl<U: Role, L: LockStatus> Terminal for CliTerminal<U, L> {
    fn prompt(&self, message: &str) -> Result<(), Box<dyn Error>> {
        println!("{}", message);
        Ok(())
    }
}

impl<U: Role, L: LockStatus> CliTerminal<U, L> {
    fn list_products(&self) -> Result<(), Box<dyn Error>> {
        self.prompt("Products:")?;
        for product in self.vending_machine.look_up() {
            self.prompt(&format!("{:?}", product))?;
        }

        Ok(())
    }

    fn exit(&self) -> Result<(), Box<dyn Error>> {
        self.prompt("Goodbye! Thanks for using the vending machine!")?;
        std::process::exit(0);
    }
}

impl<L: LockStatus> CliTerminal<Admin, L> {
    fn list_sales(&self) -> Result<(), Box<dyn Error>> {
        self.prompt("Sales report:")?;
        for sale in self.vending_machine.list_sales_report() {
            self.prompt(&format!("{:?}", sale))?;
        }

        Ok(())
    }
}

impl<U: Authenticated> CliTerminal<U, Unlocked> {
    fn logout(self) -> Result<PromptPerspective, Box<dyn Error>> {
        Ok(PromptPerspective::GuestUnlocked(CliTerminal::<
            Guest,
            Unlocked,
        >::new(
            self.vending_machine.logout(),
        )))
    }
}

impl<U: Authenticated> CliTerminal<U, Locked> {
    fn logout(self) -> Result<PromptPerspective, Box<dyn Error>> {
        Ok(PromptPerspective::GuestLocked(
            CliTerminal::<Guest, Locked> {
                vending_machine: self.vending_machine.logout(),
            },
        ))
    }
}

impl CliTerminal<Guest, Unlocked> {
    pub fn run(mut self) -> Result<PromptPerspective, Box<dyn Error>> {
        loop {
            match self.choose_command() {
                Ok(GuestUnlockedCommand::Login) => {
                    return self.login();
                }
                Ok(GuestUnlockedCommand::ListProducts) => {
                    self.list_products()?;
                }
                Ok(GuestUnlockedCommand::BuyProduct) => {
                    self.buy_product()?;
                }
                Ok(GuestUnlockedCommand::Exit) => {
                    self.exit()?;
                }
                Err(e) => {
                    self.prompt(&format!("Error: {}", e))?;
                }
            }
            self.prompt("")?;
        }
    }

    fn choose_command(&self) -> Result<GuestUnlockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:")?;
        self.prompt(&GuestUnlockedCommand::Login.to_string())?;
        self.prompt(&GuestUnlockedCommand::ListProducts.to_string())?;
        self.prompt(&GuestUnlockedCommand::BuyProduct.to_string())?;
        self.prompt(&GuestUnlockedCommand::Exit.to_string())?;

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        GuestUnlockedCommand::try_from(command.trim())
    }

    fn login(self) -> Result<PromptPerspective, Box<dyn Error>> {
        self.prompt("Enter your username:")?;
        let mut username = String::new();
        std::io::stdin().read_line(&mut username)?;

        self.prompt("Enter your password:")?;
        let mut password = String::new();
        std::io::stdin().read_line(&mut password)?;

        let username = Name::parse(username.trim())?;
        let password = Password::parse(password.trim())?;

        match self.vending_machine.login(&username, &password) {
            AuthResult::SuccessAdmin(vending_machine) => {
                Ok(PromptPerspective::AdminUnlocked(CliTerminal::<
                    Admin,
                    Unlocked,
                > {
                    vending_machine,
                }))
            }
            AuthResult::SuccessSupplier(vending_machine) => {
                Ok(PromptPerspective::SupplierUnlocked(CliTerminal::<
                    Supplier,
                    Unlocked,
                > {
                    vending_machine,
                }))
            }
            AuthResult::Failure(vending_machine) => {
                Ok(PromptPerspective::GuestUnlocked(CliTerminal::<
                    Guest,
                    Unlocked,
                > {
                    vending_machine,
                }))
            }
        }
    }

    fn buy_product(&mut self) -> Result<(), Box<dyn Error>> {
        self.prompt("Enter the product id:")?;
        let mut product_id = String::new();
        std::io::stdin().read_line(&mut product_id)?;

        let product_id = Value::parse(product_id.trim())?;

        self.prompt("Enter the amount:")?;
        let mut amount = String::new();
        std::io::stdin().read_line(&mut amount)?;

        let amount = Value::parse(amount.trim())?;

        let product = self.vending_machine.buy(product_id, amount)?;

        self.prompt(&format!("Product bought successfully: {:?}", product))?;

        Ok(())
    }
}

impl CliTerminal<Guest, Locked> {
    pub fn run(self) -> Result<PromptPerspective, Box<dyn Error>> {
        loop {
            match self.choose_command() {
                Ok(GuestLockedCommand::Login) => {
                    return self.login();
                }
                Ok(GuestLockedCommand::ListProducts) => {
                    self.list_products()?;
                }
                Ok(GuestLockedCommand::Exit) => {
                    self.exit()?;
                }
                Err(e) => {
                    self.prompt(&format!("Error: {}", e))?;
                }
            }
            self.prompt("")?;
        }
    }

    fn choose_command(&self) -> Result<GuestLockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:")?;
        self.prompt(&GuestLockedCommand::Login.to_string())?;
        self.prompt(&GuestLockedCommand::ListProducts.to_string())?;
        self.prompt(&GuestLockedCommand::Exit.to_string())?;

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        GuestLockedCommand::try_from(command.trim())
    }

    fn login(self) -> Result<PromptPerspective, Box<dyn Error>> {
        self.prompt("Enter your username:")?;
        let mut username = String::new();
        std::io::stdin().read_line(&mut username)?;

        self.prompt("Enter your password:")?;
        let mut password = String::new();
        std::io::stdin().read_line(&mut password)?;

        let username = Name::parse(username.trim())?;
        let password = Password::parse(password.trim())?;

        match self.vending_machine.login(&username, &password) {
            AuthResult::SuccessAdmin(vending_machine) => Ok(PromptPerspective::AdminLocked(
                CliTerminal::<Admin, Locked> { vending_machine },
            )),
            AuthResult::SuccessSupplier(vending_machine) => {
                Ok(PromptPerspective::SupplierLocked(CliTerminal::<
                    Supplier,
                    Locked,
                > {
                    vending_machine,
                }))
            }
            AuthResult::Failure(vending_machine) => Ok(PromptPerspective::GuestLocked(
                CliTerminal::<Guest, Locked> { vending_machine },
            )),
        }
    }
}

impl CliTerminal<Admin, Unlocked> {
    pub fn run(self) -> Result<PromptPerspective, Box<dyn Error>> {
        loop {
            match self.choose_command() {
                Ok(AdminUnlockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(AdminUnlockedCommand::ListProducts) => {
                    self.list_products()?;
                }
                Ok(AdminUnlockedCommand::ListSales) => {
                    self.list_sales()?;
                }
                Ok(AdminUnlockedCommand::Lock) => {
                    return self.lock();
                }
                Ok(AdminUnlockedCommand::Exit) => {
                    self.exit()?;
                }
                Err(e) => {
                    self.prompt(&format!("Error: {}", e))?;
                }
            }
            self.prompt("")?;
        }
    }

    fn choose_command(&self) -> Result<AdminUnlockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:")?;
        self.prompt(&AdminUnlockedCommand::Logout.to_string())?;
        self.prompt(&AdminUnlockedCommand::ListProducts.to_string())?;
        self.prompt(&AdminUnlockedCommand::ListSales.to_string())?;
        self.prompt(&AdminUnlockedCommand::Lock.to_string())?;
        self.prompt(&AdminUnlockedCommand::Exit.to_string())?;

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        AdminUnlockedCommand::try_from(command.trim())
    }

    fn lock(self) -> Result<PromptPerspective, Box<dyn Error>> {
        Ok(PromptPerspective::AdminLocked(
            CliTerminal::<Admin, Locked> {
                vending_machine: self.vending_machine.lock(),
            },
        ))
    }
}

impl CliTerminal<Admin, Locked> {
    pub fn run(self) -> Result<PromptPerspective, Box<dyn Error>> {
        loop {
            match self.choose_command() {
                Ok(AdminLockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(AdminLockedCommand::ListProducts) => {
                    self.list_products()?;
                }
                Ok(AdminLockedCommand::ListSales) => {
                    self.list_sales()?;
                }
                Ok(AdminLockedCommand::Unlock) => {
                    return self.unlock();
                }
                Ok(AdminLockedCommand::Exit) => {
                    self.exit()?;
                }
                Err(e) => {
                    self.prompt(&format!("Error: {}", e))?;
                }
            }
            self.prompt("")?;
        }
    }

    fn choose_command(&self) -> Result<AdminLockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:")?;
        self.prompt(&AdminLockedCommand::Logout.to_string())?;
        self.prompt(&AdminLockedCommand::ListProducts.to_string())?;
        self.prompt(&AdminLockedCommand::ListSales.to_string())?;
        self.prompt(&AdminLockedCommand::Unlock.to_string())?;
        self.prompt(&AdminLockedCommand::Exit.to_string())?;

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        AdminLockedCommand::try_from(command.trim())
    }

    fn unlock(self) -> Result<PromptPerspective, Box<dyn Error>> {
        Ok(PromptPerspective::AdminUnlocked(CliTerminal::<
            Admin,
            Unlocked,
        > {
            vending_machine: self.vending_machine.unlock(),
        }))
    }
}

impl CliTerminal<Supplier, Unlocked> {
    pub fn run(mut self) -> Result<PromptPerspective, Box<dyn Error>> {
        loop {
            match self.choose_command() {
                Ok(SupplierUnlockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(SupplierUnlockedCommand::ListProducts) => {
                    self.list_products()?;
                }
                Ok(SupplierUnlockedCommand::SupplyProduct) => {
                    self.supply_product()?;
                }
                Ok(SupplierUnlockedCommand::Exit) => {
                    self.exit()?;
                }
                Err(e) => {
                    self.prompt(&format!("Error: {}", e))?;
                }
            }
            self.prompt("")?;
        }
    }

    fn choose_command(&self) -> Result<SupplierUnlockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:")?;
        self.prompt(&SupplierUnlockedCommand::Logout.to_string())?;
        self.prompt(&SupplierUnlockedCommand::ListProducts.to_string())?;
        self.prompt(&SupplierUnlockedCommand::SupplyProduct.to_string())?;
        self.prompt(&SupplierUnlockedCommand::Exit.to_string())?;

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        SupplierUnlockedCommand::try_from(command.trim())
    }

    fn supply_product(&mut self) -> Result<(), Box<dyn Error>> {
        self.prompt("Enter the product id:")?;
        let mut product_id = String::new();
        std::io::stdin().read_line(&mut product_id)?;

        let product_id = Value::parse(product_id.trim())?;

        self.prompt("Enter the product name:")?;
        let mut product_name = String::new();
        std::io::stdin().read_line(&mut product_name)?;

        let product_name = Name::parse(product_name.trim())?;

        self.prompt("Enter the price:")?;
        let mut price = String::new();
        std::io::stdin().read_line(&mut price)?;

        let price = Price::parse(price.trim())?;

        self.prompt("Enter the quantity:")?;
        let mut quantity = String::new();
        std::io::stdin().read_line(&mut quantity)?;

        let quantity = Value::parse(quantity.trim())?;

        let product = Product {
            column_id: product_id,
            name: product_name,
            price,
            quantity,
        };

        self.vending_machine.supply_product(product.clone())?;

        self.prompt(&format!("Product supplied successfully: {:?}", product))?;

        Ok(())
    }
}

impl CliTerminal<Supplier, Locked> {
    pub fn run(self) -> Result<PromptPerspective, Box<dyn Error>> {
        loop {
            match self.choose_command() {
                Ok(SupplierLockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(SupplierLockedCommand::ListProducts) => {
                    self.list_products()?;
                }
                Ok(SupplierLockedCommand::Exit) => {
                    self.exit()?;
                }
                Err(e) => {
                    self.prompt(&format!("Error: {}", e))?;
                }
            }
            self.prompt("")?;
        }
    }

    fn choose_command(&self) -> Result<SupplierLockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:")?;
        self.prompt(&SupplierLockedCommand::Logout.to_string())?;
        self.prompt(&SupplierLockedCommand::ListProducts.to_string())?;
        self.prompt(&SupplierLockedCommand::Exit.to_string())?;

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        SupplierLockedCommand::try_from(command.trim())
    }
}
