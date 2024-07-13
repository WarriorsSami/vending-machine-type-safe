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

#[derive(Clone)]
pub struct CliPaymentTerminal;

impl Terminal for CliPaymentTerminal {}

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

impl<U: Role, L: LockStatus> Terminal for CliTerminal<U, L> {}

impl CliTerminal<Guest, Unlocked> {
    pub fn new(vending_machine: VendingMachine<Guest, Unlocked>) -> Self {
        Self { vending_machine }
    }
}

impl<U: Role, L: LockStatus> CliTerminal<U, L> {
    async fn list_products(&self) -> Result<(), Box<dyn Error>> {
        self.prompt("Products:");
        for product in self.vending_machine.look_up().await {
            self.prompt(&format!("{:?}", product));
        }

        Ok(())
    }

    fn exit(&self) -> Result<(), Box<dyn Error>> {
        self.prompt("Goodbye! Thanks for using the vending machine!");
        std::process::exit(0);
    }
}

impl<L: LockStatus> CliTerminal<Guest, L> {
    fn pre_login(&self) -> Result<(Name, Password), Box<dyn Error>> {
        self.prompt("Enter your username:");
        let mut username = String::new();
        std::io::stdin().read_line(&mut username)?;

        self.prompt("Enter your password:");
        let mut password = String::new();
        std::io::stdin().read_line(&mut password)?;

        let username = Name::parse(username.trim())?;
        let password = Password::parse(password.trim())?;

        Ok((username, password))
    }
}

impl<L: LockStatus> CliTerminal<Admin, L> {
    async fn list_sales(&self) -> Result<(), Box<dyn Error>> {
        self.prompt("Sales report:");
        for sale in self.vending_machine.list_sales_report().await {
            self.prompt(&format!("{:?}", sale));
        }

        Ok(())
    }
}

impl<U: Authenticated> CliTerminal<U, Unlocked> {
    fn logout(self) -> PromptPerspective {
        PromptPerspective::GuestUnlocked(CliTerminal::<Guest, Unlocked>::new(
            self.vending_machine.logout(),
        ))
    }
}

impl<U: Authenticated> CliTerminal<U, Locked> {
    fn logout(self) -> PromptPerspective {
        PromptPerspective::GuestLocked(CliTerminal::<Guest, Locked> {
            vending_machine: self.vending_machine.logout(),
        })
    }
}

impl CliTerminal<Guest, Unlocked> {
    pub async fn run(mut self) -> PromptPerspective {
        loop {
            match self.choose_command() {
                Ok(GuestUnlockedCommand::Login) => match self.pre_login() {
                    Ok((username, password)) => {
                        return self.login(username, password);
                    }
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(GuestUnlockedCommand::ListProducts) => match self.list_products().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(GuestUnlockedCommand::BuyProduct) => match self.buy_product().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(GuestUnlockedCommand::Exit) => match self.exit() {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Err(e) => {
                    self.prompt(&format!("Error: {}", e));
                }
            }
            self.prompt("");
        }
    }

    fn choose_command(&self) -> Result<GuestUnlockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:");
        self.prompt(&GuestUnlockedCommand::Login.to_string());
        self.prompt(&GuestUnlockedCommand::ListProducts.to_string());
        self.prompt(&GuestUnlockedCommand::BuyProduct.to_string());
        self.prompt(&GuestUnlockedCommand::Exit.to_string());

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        GuestUnlockedCommand::try_from(command.trim())
    }

    fn login(self, username: Name, password: Password) -> PromptPerspective {
        match self.vending_machine.login(&username, &password) {
            AuthResult::SuccessAdmin(vending_machine) => {
                PromptPerspective::AdminUnlocked(CliTerminal::<Admin, Unlocked> { vending_machine })
            }
            AuthResult::SuccessSupplier(vending_machine) => {
                PromptPerspective::SupplierUnlocked(CliTerminal::<Supplier, Unlocked> {
                    vending_machine,
                })
            }
            AuthResult::Failure(vending_machine) => {
                PromptPerspective::GuestUnlocked(CliTerminal::<Guest, Unlocked> { vending_machine })
            }
        }
    }

    async fn buy_product(&mut self) -> Result<(), Box<dyn Error>> {
        self.prompt("Enter the product id:");
        let mut product_id = String::new();
        std::io::stdin().read_line(&mut product_id)?;

        let product_id = Value::parse(product_id.trim())?;

        self.prompt("Enter the amount:");
        let mut amount = String::new();
        std::io::stdin().read_line(&mut amount)?;

        let amount = Value::parse(amount.trim())?;

        let product = self.vending_machine.buy(product_id, amount).await?;

        self.prompt(&format!("Product bought successfully: {:?}", product));

        Ok(())
    }
}

impl CliTerminal<Guest, Locked> {
    pub async fn run(self) -> PromptPerspective {
        loop {
            match self.choose_command() {
                Ok(GuestLockedCommand::Login) => match self.pre_login() {
                    Ok((username, password)) => {
                        return self.login(username, password);
                    }
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(GuestLockedCommand::ListProducts) => match self.list_products().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(GuestLockedCommand::Exit) => match self.exit() {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Err(e) => {
                    self.prompt(&format!("Error: {}", e));
                }
            }
            self.prompt("");
        }
    }

    fn choose_command(&self) -> Result<GuestLockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:");
        self.prompt(&GuestLockedCommand::Login.to_string());
        self.prompt(&GuestLockedCommand::ListProducts.to_string());
        self.prompt(&GuestLockedCommand::Exit.to_string());

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        GuestLockedCommand::try_from(command.trim())
    }

    fn login(self, username: Name, password: Password) -> PromptPerspective {
        match self.vending_machine.login(&username, &password) {
            AuthResult::SuccessAdmin(vending_machine) => {
                PromptPerspective::AdminLocked(CliTerminal::<Admin, Locked> { vending_machine })
            }
            AuthResult::SuccessSupplier(vending_machine) => {
                PromptPerspective::SupplierLocked(CliTerminal::<Supplier, Locked> {
                    vending_machine,
                })
            }
            AuthResult::Failure(vending_machine) => {
                PromptPerspective::GuestLocked(CliTerminal::<Guest, Locked> { vending_machine })
            }
        }
    }
}

impl CliTerminal<Admin, Unlocked> {
    pub async fn run(self) -> PromptPerspective {
        loop {
            match self.choose_command() {
                Ok(AdminUnlockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(AdminUnlockedCommand::ListProducts) => match self.list_products().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(AdminUnlockedCommand::ListSales) => match self.list_sales().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(AdminUnlockedCommand::Lock) => {
                    return self.lock();
                }
                Ok(AdminUnlockedCommand::Exit) => match self.exit() {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Err(e) => {
                    self.prompt(&format!("Error: {}", e));
                }
            }
            self.prompt("");
        }
    }

    fn choose_command(&self) -> Result<AdminUnlockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:");
        self.prompt(&AdminUnlockedCommand::Logout.to_string());
        self.prompt(&AdminUnlockedCommand::ListProducts.to_string());
        self.prompt(&AdminUnlockedCommand::ListSales.to_string());
        self.prompt(&AdminUnlockedCommand::Lock.to_string());
        self.prompt(&AdminUnlockedCommand::Exit.to_string());

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        AdminUnlockedCommand::try_from(command.trim())
    }

    fn lock(self) -> PromptPerspective {
        PromptPerspective::AdminLocked(CliTerminal::<Admin, Locked> {
            vending_machine: self.vending_machine.lock(),
        })
    }
}

impl CliTerminal<Admin, Locked> {
    pub async fn run(self) -> PromptPerspective {
        loop {
            match self.choose_command() {
                Ok(AdminLockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(AdminLockedCommand::ListProducts) => match self.list_products().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(AdminLockedCommand::ListSales) => match self.list_sales().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(AdminLockedCommand::Unlock) => {
                    return self.unlock();
                }
                Ok(AdminLockedCommand::Exit) => match self.exit() {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Err(e) => {
                    self.prompt(&format!("Error: {}", e));
                }
            }
            self.prompt("");
        }
    }

    fn choose_command(&self) -> Result<AdminLockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:");
        self.prompt(&AdminLockedCommand::Logout.to_string());
        self.prompt(&AdminLockedCommand::ListProducts.to_string());
        self.prompt(&AdminLockedCommand::ListSales.to_string());
        self.prompt(&AdminLockedCommand::Unlock.to_string());
        self.prompt(&AdminLockedCommand::Exit.to_string());

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        AdminLockedCommand::try_from(command.trim())
    }

    fn unlock(self) -> PromptPerspective {
        PromptPerspective::AdminUnlocked(CliTerminal::<Admin, Unlocked> {
            vending_machine: self.vending_machine.unlock(),
        })
    }
}

impl CliTerminal<Supplier, Unlocked> {
    pub async fn run(mut self) -> PromptPerspective {
        loop {
            match self.choose_command() {
                Ok(SupplierUnlockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(SupplierUnlockedCommand::ListProducts) => match self.list_products().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(SupplierUnlockedCommand::SupplyProduct) => match self.supply_product().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(SupplierUnlockedCommand::Exit) => match self.exit() {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Err(e) => {
                    self.prompt(&format!("Error: {}", e));
                }
            }
            self.prompt("");
        }
    }

    fn choose_command(&self) -> Result<SupplierUnlockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:");
        self.prompt(&SupplierUnlockedCommand::Logout.to_string());
        self.prompt(&SupplierUnlockedCommand::ListProducts.to_string());
        self.prompt(&SupplierUnlockedCommand::SupplyProduct.to_string());
        self.prompt(&SupplierUnlockedCommand::Exit.to_string());

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        SupplierUnlockedCommand::try_from(command.trim())
    }

    async fn supply_product(&mut self) -> Result<(), Box<dyn Error>> {
        self.prompt("Enter the product id:");
        let mut product_id = String::new();
        std::io::stdin().read_line(&mut product_id)?;

        let product_id = Value::parse(product_id.trim())?;

        self.prompt("Enter the product name:");
        let mut product_name = String::new();
        std::io::stdin().read_line(&mut product_name)?;

        let product_name = Name::parse(product_name.trim())?;

        self.prompt("Enter the price:");
        let mut price = String::new();
        std::io::stdin().read_line(&mut price)?;

        let price = Price::parse(price.trim())?;

        self.prompt("Enter the quantity:");
        let mut quantity = String::new();
        std::io::stdin().read_line(&mut quantity)?;

        let quantity = Value::parse(quantity.trim())?;

        let product = Product {
            column_id: product_id,
            name: product_name,
            price,
            quantity,
        };

        self.vending_machine.supply_product(product.clone()).await?;

        self.prompt(&format!("Product supplied successfully: {:?}", product));

        Ok(())
    }
}

impl CliTerminal<Supplier, Locked> {
    pub async fn run(self) -> PromptPerspective {
        loop {
            match self.choose_command() {
                Ok(SupplierLockedCommand::Logout) => {
                    return self.logout();
                }
                Ok(SupplierLockedCommand::ListProducts) => match self.list_products().await {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Ok(SupplierLockedCommand::Exit) => match self.exit() {
                    Ok(_) => {}
                    Err(e) => {
                        self.prompt(&format!("Error: {}", e));
                    }
                },
                Err(e) => {
                    self.prompt(&format!("Error: {}", e));
                }
            }
            self.prompt("");
        }
    }

    fn choose_command(&self) -> Result<SupplierLockedCommand, Box<dyn Error>> {
        self.prompt("Choose a command:");
        self.prompt(&SupplierLockedCommand::Logout.to_string());
        self.prompt(&SupplierLockedCommand::ListProducts.to_string());
        self.prompt(&SupplierLockedCommand::Exit.to_string());

        let mut command = String::new();
        std::io::stdin().read_line(&mut command)?;

        SupplierLockedCommand::try_from(command.trim())
    }
}
