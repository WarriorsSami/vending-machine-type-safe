pub mod entities {
    #[derive(Clone, Debug)]
    pub struct Name(String);

    impl Name {
        pub fn parse(value: &str) -> Result<Self, String> {
            if value.is_empty() {
                return Err("Name cannot be empty".to_string());
            }

            if value.len() > 30 {
                return Err("Name is too long".to_string());
            }

            Ok(Self(value.to_string()))
        }
    }

    impl AsRef<str> for Name {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    #[derive(Clone, Debug)]
    pub struct Password(String);

    impl Password {
        pub fn parse(value: &str) -> Result<Self, String> {
            if value.is_empty() {
                return Err("Password cannot be empty".to_string());
            }

            if value.len() < 8 {
                return Err("Password is too short".to_string());
            }

            Ok(Self(value.to_string()))
        }
    }

    impl AsRef<str> for Password {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    #[derive(Clone, Debug)]
    pub struct Price(f32);

    impl Price {
        pub fn parse(value: &str) -> Result<Self, String> {
            let value = value
                .parse::<f32>()
                .map_err(|_| "Price must be a number".to_string())?;

            if value <= 0.0 {
                return Err("Price must be greater than zero".to_string());
            }

            Ok(Self(value))
        }
    }

    impl AsRef<f32> for Price {
        fn as_ref(&self) -> &f32 {
            &self.0
        }
    }

    #[derive(Clone, Debug)]
    pub struct Product {
        pub column_id: u32,
        pub name: Name,
        pub price: Price,
        pub quantity: u32,
    }
}

pub mod repositories {
    use super::entities::Product;

    pub trait ProductRepository {
        fn find(&self, column_id: u32) -> Option<Product>;
        fn save(&mut self, product: Product) -> Result<(), Box<dyn std::error::Error>>;
        fn find_all(&self) -> &Vec<Product>;
    }
}
