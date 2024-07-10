use crate::application::auth::*;
use crate::domain::entities::{Name, Password, Price, Product};
use crate::domain::repositories;

pub mod auth {
    use crate::application::VendingMachine;

    pub trait Role {}

    pub struct Guest;
    pub struct Admin;

    impl Role for Guest {}
    impl Role for Admin {}

    pub enum AuthResult {
        Success(VendingMachine<Admin>),
        Failure(VendingMachine<Guest>),
    }
}

pub struct VendingMachine<U: Role> {
    product_repository: Box<dyn repositories::ProductRepository>,
    _role: std::marker::PhantomData<U>,
}

impl<U: Role> VendingMachine<U> {
    pub fn new(
        product_repository: Box<dyn repositories::ProductRepository>,
    ) -> VendingMachine<Guest> {
        VendingMachine::<Guest> {
            product_repository,
            _role: std::marker::PhantomData,
        }
    }

    pub fn look_up(&self) -> &Vec<Product> {
        self.product_repository.find_all()
    }
}

impl VendingMachine<Guest> {
    pub fn login(self, username: &Name, password: &Password) -> AuthResult {
        match (username.as_ref(), password.as_ref()) {
            ("admin", "admin_pass") => AuthResult::Success(VendingMachine::<Admin> {
                product_repository: self.product_repository,
                _role: std::marker::PhantomData,
            }),
            _ => AuthResult::Failure(self),
        }
    }

    fn pay(&self, amount: Price) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    pub fn buy(&self, column_id: u32) -> Result<&Product, Box<dyn std::error::Error>> {
        todo!()
    }
}

impl VendingMachine<Admin> {
    pub fn logout(self) -> VendingMachine<Guest> {
        VendingMachine {
            product_repository: self.product_repository,
            _role: std::marker::PhantomData,
        }
    }

    pub fn list_sales_report(&self) {
        todo!()
    }

    pub fn list_stock_report(&self) {
        todo!()
    }

    pub fn list_volume_report(&self) {
        todo!()
    }

    pub fn supply_existing_product(
        &mut self,
        column_id: u32,
        amount: u32,
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
