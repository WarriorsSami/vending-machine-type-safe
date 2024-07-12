use vending_machine::domain::entities::{Product, Sale, Value};
use vending_machine::domain::interfaces::{ProductRepository, SaleRepository};

#[derive(Default)]
pub struct InMemoryProductRepository {
    products: Vec<Product>,
}

impl ProductRepository for InMemoryProductRepository {
    fn find(&self, column_id: Value) -> Option<Product> {
        self.products
            .iter()
            .find(|product| product.column_id == column_id)
            .cloned()
    }

    fn save(&mut self, product: Product) -> Result<(), Box<dyn std::error::Error>> {
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

    fn find_all(&self) -> &Vec<Product> {
        self.products.as_ref()
    }
}

#[derive(Default)]
pub struct InMemorySaleRepository {
    sales: Vec<Sale>,
}

impl SaleRepository for InMemorySaleRepository {
    fn save(&mut self, sale: Sale) -> Result<(), Box<dyn std::error::Error>> {
        self.sales.push(sale);
        Ok(())
    }

    fn find_all(&self) -> &Vec<Sale> {
        self.sales.as_ref()
    }
}
