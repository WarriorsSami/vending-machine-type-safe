use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use infrastructure::in_memory::{InMemoryProductRepository, InMemorySaleRepository};
use infrastructure::sqlite::{SqliteProductRepository, SqliteSaleRepository};
use sqlx::SqlitePool;
use vending_machine::application::states::{Guest, Unlocked};
use vending_machine::application::VendingMachine;
use vending_machine::domain::interfaces::{PaymentTerminal, ProductRepository, SaleRepository};

use crate::terminals::{CliPaymentTerminal, CliTerminal};

struct TypeMap(HashMap<TypeId, Box<dyn Any>>);

impl TypeMap {
    pub fn set<T>(&mut self, t: T)
    where
        T: Any + 'static,
    {
        self.0.insert(TypeId::of::<T>(), Box::new(t));
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Any + 'static,
    {
        self.0
            .get(&TypeId::of::<T>())
            .map(|boxed| boxed.downcast_ref::<T>().unwrap())
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any + 'static,
    {
        self.0
            .get_mut(&TypeId::of::<T>())
            .map(|boxed| boxed.downcast_mut::<T>().unwrap())
    }

    pub fn has<T>(&self) -> bool
    where
        T: Any + 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }
}

#[async_trait]
pub trait DIBuilder {
    type Input;
    type Output;

    async fn build(input: Self::Input) -> Self::Output;
}

#[async_trait]
impl DIBuilder for InMemoryProductRepository {
    type Input = ();
    type Output = Box<dyn ProductRepository>;

    async fn build(_: Self::Input) -> Self::Output {
        Box::new(InMemoryProductRepository::default())
    }
}

#[async_trait]
impl DIBuilder for InMemorySaleRepository {
    type Input = ();
    type Output = Box<dyn SaleRepository>;

    async fn build(_: Self::Input) -> Self::Output {
        Box::new(InMemorySaleRepository::default())
    }
}

#[async_trait]
impl DIBuilder for SqlitePool {
    type Input = ();
    type Output = Self;

    async fn build(_: Self::Input) -> Self::Output {
        let dsn = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        SqlitePool::connect(&dsn)
            .await
            .expect("Failed to connect to database")
    }
}

#[async_trait]
impl DIBuilder for SqliteProductRepository {
    type Input = (DIObj<SqlitePool>, ());
    type Output = Box<dyn ProductRepository>;

    async fn build((pool, _): Self::Input) -> Self::Output {
        let pool = pool.lock().unwrap().clone();

        Box::new(SqliteProductRepository::new(pool))
    }
}

#[async_trait]
impl DIBuilder for SqliteSaleRepository {
    type Input = (DIObj<SqlitePool>, ());
    type Output = Box<dyn SaleRepository>;

    async fn build((pool, _): Self::Input) -> Self::Output {
        let pool = pool.lock().unwrap().clone();

        Box::new(SqliteSaleRepository::new(pool))
    }
}

#[async_trait]
impl DIBuilder for CliPaymentTerminal {
    type Input = ();
    type Output = Box<dyn PaymentTerminal>;

    async fn build(_: Self::Input) -> Self::Output {
        Box::new(CliPaymentTerminal)
    }
}

#[async_trait]
impl DIBuilder for VendingMachine<Guest, Unlocked> {
    type Input = (
        DIObj<Box<dyn ProductRepository>>,
        (
            DIObj<Box<dyn SaleRepository>>,
            (DIObj<Box<dyn PaymentTerminal>>, ()),
        ),
    );
    type Output = Self;

    async fn build(
        (product_repository, (sale_repository, (payment_terminal, _))): Self::Input,
    ) -> Self::Output {
        let product_repository = product_repository.lock().unwrap().clone();
        let sale_repository = sale_repository.lock().unwrap().clone();
        let payment_terminal = payment_terminal.lock().unwrap().clone();

        VendingMachine::new(product_repository, sale_repository, payment_terminal)
    }
}

#[async_trait]
impl DIBuilder for CliTerminal<Guest, Unlocked> {
    type Input = (DIObj<VendingMachine<Guest, Unlocked>>, ());
    type Output = Self;

    async fn build((vending_machine, _): Self::Input) -> Self::Output {
        let vending_machine = vending_machine.lock().unwrap().clone();

        CliTerminal::new(vending_machine)
    }
}

type DIObj<T> = Arc<Mutex<T>>;

pub struct DIManager(TypeMap);

impl DIManager {
    pub fn new() -> Self {
        Self(TypeMap(HashMap::new()))
    }

    pub async fn build<T>(&mut self) -> Option<DIObj<T::Output>>
    where
        T: DIBuilder,
        <T as DIBuilder>::Input: GetInput,
        <T as DIBuilder>::Output: 'static,
    {
        let input = T::Input::get_input(self)?;
        let obj = T::build(input).await;
        let sync_obj = Arc::new(Mutex::new(obj));
        self.0.set::<DIObj<T::Output>>(sync_obj.clone());
        Some(sync_obj)
    }
}

pub trait GetInput: Sized {
    fn get_input(manager: &DIManager) -> Option<Self>;
}

impl<T: 'static> GetInput for DIObj<T> {
    fn get_input(manager: &DIManager) -> Option<Self> {
        manager.0.get::<Self>().cloned()
    }
}

impl GetInput for () {
    fn get_input(_: &DIManager) -> Option<Self> {
        Some(())
    }
}

impl<S, T> GetInput for (S, T)
where
    S: GetInput,
    T: GetInput,
{
    fn get_input(manager: &DIManager) -> Option<Self> {
        S::get_input(manager).and_then(|s| T::get_input(manager).map(|t| (s, t)))
    }
}
