use async_trait::async_trait;
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::SqlitePool;
use yadir::core::contracts::DIBuilder;
use yadir::core::primitives::DIObj;
use yadir::{deps, DIBuilder};

use vending_machine::domain::entities::{Name, Price, Product, Sale, Value};
use vending_machine::domain::interfaces::{ProductRepository, SaleRepository};

#[derive(Clone, DIBuilder)]
#[build_method("default")]
pub struct DbConn(SqlitePool);

impl Default for DbConn {
    fn default() -> Self {
        let dsn = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = SqlitePool::connect(&dsn);

        futures::executor::block_on(async {
            DbConn(pool.await.expect("Failed to connect to database"))
        })
    }
}

#[derive(Default, Clone, DIBuilder)]
#[build_as(Box<dyn ProductRepository>)]
#[build_method("default")]
pub struct SqliteProductRepository {
    #[deps]
    pool: DbConn,
}

struct RawProduct {
    column_id: i64,
    name: String,
    price: f64,
    quantity: i64,
}

impl TryFrom<RawProduct> for Product {
    type Error = Box<dyn std::error::Error>;

    fn try_from(raw: RawProduct) -> Result<Self, Self::Error> {
        Ok(Product {
            column_id: Value::parse_i32(raw.column_id as i32)?,
            name: Name::parse(&raw.name)?,
            price: Price::parse_f32(raw.price as f32)?,
            quantity: Value::parse_i32(raw.quantity as i32)?,
        })
    }
}

#[async_trait]
impl ProductRepository for SqliteProductRepository {
    async fn find(&self, column_id: Value) -> Option<Product> {
        let column_id = column_id.as_value() as i32;

        let product = sqlx::query_as!(
            RawProduct,
            r#"SELECT column_id, name, price, quantity FROM product WHERE column_id = ?"#,
            column_id
        )
        .fetch_one(&self.pool.0)
        .await
        .ok()?;

        Some(Product::try_from(product).ok()?)
    }

    async fn save(&mut self, product: Product) -> Result<(), Box<dyn std::error::Error>> {
        let existing_product = self.find(product.column_id.clone()).await;
        match existing_product {
            Some(_) => {
                let name = product.name.clone().as_ref().to_string();
                let price = product.price.clone().as_value();
                let quantity = product.quantity.clone().as_value() as i32;
                let column_id = product.column_id.clone().as_value() as i32;

                sqlx::query!(
                    r#"UPDATE product SET name = ?, price = ?, quantity = ? WHERE column_id = ?"#,
                    name,
                    price,
                    quantity,
                    column_id
                )
                .execute(&self.pool.0)
                .await?;
            }
            None => {
                let name = product.name.clone().as_ref().to_string();
                let price = product.price.clone().as_value();
                let quantity = product.quantity.clone().as_value() as i32;
                let column_id = product.column_id.clone().as_value() as i32;

                sqlx::query!(
                    r#"INSERT INTO product (column_id, name, price, quantity) VALUES (?, ?, ?, ?)"#,
                    column_id,
                    name,
                    price,
                    quantity
                )
                .execute(&self.pool.0)
                .await?;
            }
        }

        Ok(())
    }

    async fn find_all(&self) -> Vec<Product> {
        let products = sqlx::query_as!(
            RawProduct,
            r#"SELECT column_id, name, price, quantity FROM product"#
        )
        .fetch_all(&self.pool.0)
        .await
        .unwrap_or(vec![]);

        products
            .into_iter()
            .map(|product| product.try_into())
            .filter_map(Result::ok)
            .collect()
    }
}

#[derive(Default, Clone, DIBuilder)]
#[build_as(Box<dyn SaleRepository>)]
#[build_method("default")]
pub struct SqliteSaleRepository {
    #[deps]
    pool: DbConn,
}

struct RawSale {
    id: i64,
    date: NaiveDateTime,
    price: f64,
    product_id: i64,
}

struct ProductSalePair(RawProduct, RawSale);

impl TryFrom<ProductSalePair> for Sale {
    type Error = Box<dyn std::error::Error>;

    fn try_from(pair: ProductSalePair) -> Result<Self, Self::Error> {
        let (product, sale) = (pair.0, pair.1);
        Ok(Sale {
            date: DateTime::<Utc>::from_naive_utc_and_offset(sale.date, Utc),
            product_name: Name::parse(product.name.as_str())?,
            price: Price::parse_f32(sale.price as f32)?,
        })
    }
}

#[async_trait]
impl SaleRepository for SqliteSaleRepository {
    async fn save(&mut self, sale: Sale) -> Result<(), Box<dyn std::error::Error>> {
        let product_name = sale.product_name.clone().as_ref().to_string();

        let product = sqlx::query_as!(
            RawProduct,
            r#"SELECT column_id, name, price, quantity FROM product WHERE name = ?"#,
            product_name
        )
        .fetch_one(&self.pool.0)
        .await?;

        let product_id = product.column_id;
        let price = sale.price.clone().as_value();

        sqlx::query!(
            r#"INSERT INTO sale (date, price, product_id) VALUES (?, ?, ?)"#,
            sale.date,
            price,
            product_id
        )
        .execute(&self.pool.0)
        .await?;

        Ok(())
    }

    async fn find_all(&self) -> Vec<Sale> {
        let raw_sales = sqlx::query_as!(RawSale, r#"SELECT id, date, price, product_id FROM sale"#)
            .fetch_all(&self.pool.0)
            .await
            .unwrap_or(vec![]);

        let mut sales = Vec::<Sale>::new();
        for sale in raw_sales {
            let product = sqlx::query_as!(
                RawProduct,
                r#"SELECT column_id, name, price, quantity FROM product WHERE column_id = ?"#,
                sale.product_id
            )
            .fetch_one(&self.pool.0)
            .await;

            if let Ok(product) = product {
                let pair = ProductSalePair(product, sale);
                match Sale::try_from(pair) {
                    Ok(sale) => sales.push(sale),
                    Err(_) => continue,
                }
            }
        }

        sales
    }
}
