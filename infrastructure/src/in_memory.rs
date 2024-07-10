use vending_machine::domain::entities::Product;
use vending_machine::domain::repositories::ProductRepository;

pub struct InMemoryProductRepository {
    products: Vec<Product>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self { products: vec![] }
    }
}

impl ProductRepository for InMemoryProductRepository {
    fn find(&self, column_id: u32) -> Option<Product> {
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
