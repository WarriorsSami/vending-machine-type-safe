use async_trait::async_trait;
use vending_machine::domain::entities::{Product, Sale, Value};
use vending_machine::domain::interfaces::{ProductRepository, SaleRepository};

#[derive(Default)]
pub struct InMemoryProductRepository {
    products: Vec<Product>,
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn find(&self, column_id: Value) -> Option<Product> {
        self.products
            .iter()
            .find(|product| product.column_id == column_id)
            .cloned()
    }

    async fn save(&mut self, product: Product) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(index) = self
            .products
            .iter()
            .position(|p| p.column_id == product.column_id)
        {
            self.products[index] = product;
        } else {
            self.products.push(product);
        }

        Ok(())
    }

    async fn find_all(&self) -> Vec<Product> {
        self.products.clone()
    }
}

#[derive(Default)]
pub struct InMemorySaleRepository {
    sales: Vec<Sale>,
}

#[async_trait]
impl SaleRepository for InMemorySaleRepository {
    async fn save(&mut self, sale: Sale) -> Result<(), Box<dyn std::error::Error>> {
        self.sales.push(sale);
        Ok(())
    }

    async fn find_all(&self) -> Vec<Sale> {
        self.sales.clone()
    }
}
