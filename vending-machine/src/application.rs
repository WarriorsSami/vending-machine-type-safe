use async_trait::async_trait;
use yadir::core::contracts::DIBuilder;
use yadir::core::primitives::DIObj;
use yadir::{deps, let_deps};

use crate::application::states::*;
use crate::domain::entities::{Name, Password, Price, Product, Sale, Value};
use crate::domain::interfaces::{PaymentTerminal, ProductRepository, SaleRepository};

pub mod states {
    use dyn_clone::{clone_trait_object, DynClone};

    use crate::application::VendingMachine;

    clone_trait_object!(Role);
    clone_trait_object!(LockStatus);

    pub trait Role: Send + Sync + DynClone {}
    pub trait Authenticated: Role {}

    #[derive(Clone)]
    pub struct Guest;

    #[derive(Clone)]
    pub struct Admin;

    #[derive(Clone)]
    pub struct Supplier;

    impl Role for Guest {}
    impl Role for Admin {}
    impl Role for Supplier {}

    impl Authenticated for Admin {}
    impl Authenticated for Supplier {}

    pub enum AuthResult<L: LockStatus> {
        SuccessAdmin(VendingMachine<Admin, L>),
        SuccessSupplier(VendingMachine<Supplier, L>),
        Failure(VendingMachine<Guest, L>),
    }

    pub trait LockStatus: Send + Sync + DynClone {}

    #[derive(Clone)]
    pub struct Locked;

    #[derive(Clone)]
    pub struct Unlocked;

    impl LockStatus for Locked {}
    impl LockStatus for Unlocked {}
}

#[derive(Clone)]
pub struct VendingMachine<U: Role, L: LockStatus> {
    product_repository: Box<dyn ProductRepository>,
    sale_repository: Box<dyn SaleRepository>,
    payment_terminal: Box<dyn PaymentTerminal>,
    _role: std::marker::PhantomData<U>,
    _lock: std::marker::PhantomData<L>,
}

#[async_trait]
impl DIBuilder for VendingMachine<Guest, Unlocked> {
    type Input = deps!(
        Box<dyn ProductRepository>,
        Box<dyn SaleRepository>,
        Box<dyn PaymentTerminal>
    );
    type Output = Self;

    async fn build(input: Self::Input) -> Self::Output {
        let_deps!(product_repository, sale_repository, payment_terminal <- input);

        VendingMachine::new(product_repository, sale_repository, payment_terminal)
    }
}

impl<U: Role, L: LockStatus> VendingMachine<U, L> {
    pub async fn look_up(&self) -> Vec<Product> {
        self.product_repository.find_all().await
    }
}

impl<U: Authenticated, L: LockStatus> VendingMachine<U, L> {
    pub fn logout(self) -> VendingMachine<Guest, L> {
        VendingMachine::<Guest, L> {
            product_repository: self.product_repository,
            sale_repository: self.sale_repository,
            payment_terminal: self.payment_terminal,
            _role: std::marker::PhantomData,
            _lock: std::marker::PhantomData,
        }
    }
}

impl<L: LockStatus> VendingMachine<Guest, L> {
    pub fn login(self, username: &Name, password: &Password) -> AuthResult<L> {
        match (username.as_ref(), password.as_ref()) {
            ("admin", "admin_pass") => AuthResult::SuccessAdmin(VendingMachine::<Admin, L> {
                product_repository: self.product_repository,
                sale_repository: self.sale_repository,
                payment_terminal: self.payment_terminal,
                _role: std::marker::PhantomData,
                _lock: std::marker::PhantomData,
            }),
            ("supplier", "supplier_pass") => {
                AuthResult::SuccessSupplier(VendingMachine::<Supplier, L> {
                    product_repository: self.product_repository,
                    sale_repository: self.sale_repository,
                    payment_terminal: self.payment_terminal,
                    _role: std::marker::PhantomData,
                    _lock: std::marker::PhantomData,
                })
            }
            _ => AuthResult::Failure(self),
        }
    }
}

impl VendingMachine<Guest, Unlocked> {
    pub fn new(
        product_repository: Box<dyn ProductRepository>,
        sale_repository: Box<dyn SaleRepository>,
        payment_terminal: Box<dyn PaymentTerminal>,
    ) -> VendingMachine<Guest, Unlocked> {
        VendingMachine::<Guest, Unlocked> {
            product_repository,
            sale_repository,
            payment_terminal,
            _role: std::marker::PhantomData,
            _lock: std::marker::PhantomData,
        }
    }

    fn pay(&self, amount: Price) -> Result<(), Box<dyn std::error::Error>> {
        let mut payed_amount = Price::default();
        self.payment_terminal
            .prompt(format!("You have to pay: {}", amount.as_value()).as_str());

        loop {
            match self.payment_terminal.request() {
                Ok(value) => {
                    payed_amount = Price::parse_f32(payed_amount.as_value() + value.as_value())?;
                    if payed_amount.as_value() >= amount.as_value() {
                        let refund = Price::parse_f32(payed_amount.as_value() - amount.as_value())?;
                        self.payment_terminal.refund(refund)?;
                        return Ok(());
                    } else {
                        self.payment_terminal.prompt(
                            format!(
                                "You have to pay: {} more",
                                amount.as_value() - payed_amount.as_value()
                            )
                            .as_str(),
                        );
                    }
                }
                Err(_) => continue,
            }
        }
    }

    pub async fn buy(
        &mut self,
        column_id: Value,
        qty: Value,
    ) -> Result<Product, Box<dyn std::error::Error>> {
        let product = self
            .product_repository
            .find(column_id)
            .await
            .ok_or("Product not found")?;

        let total_price =
            Price::parse_f32(product.price.clone().as_value() * qty.as_value() as f32)?;

        let new_qty =
            Value::parse_i32(product.quantity.clone().as_value() as i32 - qty.as_value() as i32)
                .map_err(|_| "Insufficient quantity in stock")?;

        self.pay(total_price.clone())?;

        let bought_product = Product {
            quantity: new_qty,
            ..product.clone()
        };
        self.product_repository.save(bought_product.clone()).await?;

        self.sale_repository
            .save(Sale {
                product_name: product.name.clone(),
                price: Price::parse_f32(total_price.as_value())?,
                date: chrono::Utc::now(),
            })
            .await?;

        Ok(bought_product)
    }
}

impl<L: LockStatus> VendingMachine<Admin, L> {
    pub async fn list_sales_report(&self) -> Vec<Sale> {
        self.sale_repository.find_all().await
    }
}

impl VendingMachine<Admin, Unlocked> {
    pub fn lock(self) -> VendingMachine<Admin, Locked> {
        VendingMachine::<Admin, Locked> {
            product_repository: self.product_repository,
            sale_repository: self.sale_repository,
            payment_terminal: self.payment_terminal,
            _role: std::marker::PhantomData,
            _lock: std::marker::PhantomData,
        }
    }
}

impl VendingMachine<Admin, Locked> {
    pub fn unlock(self) -> VendingMachine<Admin, Unlocked> {
        VendingMachine::<Admin, Unlocked> {
            product_repository: self.product_repository,
            sale_repository: self.sale_repository,
            payment_terminal: self.payment_terminal,
            _role: std::marker::PhantomData,
            _lock: std::marker::PhantomData,
        }
    }
}

impl VendingMachine<Supplier, Unlocked> {
    pub async fn supply_product(
        &mut self,
        product: Product,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.product_repository.save(product).await
    }
}
