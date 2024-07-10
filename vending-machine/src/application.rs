use crate::application::auth::*;
use crate::domain::entities::{Name, Password, Price, Product, Value};
use crate::domain::interfaces;

pub mod auth {
    use crate::application::VendingMachine;

    pub trait Role {}
    pub trait Authenticated: Role {}

    pub struct Guest;
    pub struct Admin;
    pub struct Supplier;

    impl Role for Guest {}
    impl Role for Admin {}
    impl Role for Supplier {}

    impl Authenticated for Admin {}
    impl Authenticated for Supplier {}

    pub enum AuthResult {
        SuccessAdmin(VendingMachine<Admin>),
        SuccessSupplier(VendingMachine<Supplier>),
        Failure(VendingMachine<Guest>),
    }
}

pub struct VendingMachine<U: Role> {
    product_repository: Box<dyn interfaces::ProductRepository>,
    payment_terminal: Box<dyn interfaces::PaymentTerminal>,
    _role: std::marker::PhantomData<U>,
}

impl<U: Role> VendingMachine<U> {
    pub fn look_up(&self) -> &Vec<Product> {
        self.product_repository.find_all()
    }
}

impl<U: Authenticated> VendingMachine<U> {
    pub fn logout(self) -> VendingMachine<Guest> {
        VendingMachine::<Guest> {
            product_repository: self.product_repository,
            payment_terminal: self.payment_terminal,
            _role: std::marker::PhantomData,
        }
    }
}

impl VendingMachine<Guest> {
    pub fn new(
        product_repository: Box<dyn interfaces::ProductRepository>,
        payment_terminal: Box<dyn interfaces::PaymentTerminal>,
    ) -> VendingMachine<Guest> {
        VendingMachine::<Guest> {
            product_repository,
            payment_terminal,
            _role: std::marker::PhantomData,
        }
    }

    pub fn login(self, username: &Name, password: &Password) -> AuthResult {
        match (username.as_ref(), password.as_ref()) {
            ("admin", "admin_pass") => AuthResult::SuccessAdmin(VendingMachine::<Admin> {
                product_repository: self.product_repository,
                payment_terminal: self.payment_terminal,
                _role: std::marker::PhantomData,
            }),
            ("supplier", "supplier_pass") => {
                AuthResult::SuccessSupplier(VendingMachine::<Supplier> {
                    product_repository: self.product_repository,
                    payment_terminal: self.payment_terminal,
                    _role: std::marker::PhantomData,
                })
            }
            _ => AuthResult::Failure(self),
        }
    }

    fn pay(&self, amount: Price) -> Result<(), Box<dyn std::error::Error>> {
        let mut payed_amount = Price::default();
        self.payment_terminal
            .prompt(format!("You have to pay: {}", amount.as_value()).as_str())?;

        loop {
            match self.payment_terminal.request() {
                Ok(value) => {
                    payed_amount = payed_amount + value;
                    if payed_amount.as_value() >= amount.as_value() {
                        self.payment_terminal.refund(payed_amount - amount)?;
                        return Ok(());
                    } else {
                        self.payment_terminal.prompt(
                            format!(
                                "You have to pay: {} more",
                                amount.as_value() - payed_amount.as_value()
                            )
                            .as_str(),
                        )?;
                    }
                }
                Err(_) => continue,
            }
        }
    }

    pub fn buy(
        &mut self,
        column_id: Value,
        qty: Value,
    ) -> Result<Product, Box<dyn std::error::Error>> {
        let product = self
            .product_repository
            .find(column_id)
            .ok_or("Product not found")?;

        let total_price = product.price.clone() * qty.as_value();

        self.pay(total_price)?;

        self.product_repository.save(Product {
            quantity: product.quantity.clone() - qty,
            ..product.clone()
        })?;

        Ok(product)
    }
}

impl VendingMachine<Admin> {
    pub fn list_sales_report(&self) {
        todo!()
    }

    pub fn list_stock_report(&self) {
        todo!()
    }

    pub fn list_volume_report(&self) {
        todo!()
    }
}

impl VendingMachine<Supplier> {
    pub fn supply_existing_product(
        &mut self,
        column_id: Value,
        amount: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let product = self
            .product_repository
            .find(column_id)
            .ok_or("Product not found")?;

        self.product_repository.save(Product {
            quantity: product.quantity + amount,
            ..product
        })
    }

    pub fn supply_new_product(
        &mut self,
        product: Product,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.product_repository.save(product)
    }
}
