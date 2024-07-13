pub mod entities {
    use chrono::{DateTime, Utc};

    #[derive(Clone, Debug)]
    pub struct Name(String);

    impl Name {
        pub fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
            if value.is_empty() {
                return Err(Box::from("Name cannot be empty"));
            }

            if value.len() > 30 {
                return Err(Box::from("Name is too long"));
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
        pub fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
            if value.is_empty() {
                return Err(Box::from("Password cannot be empty"));
            }

            if value.len() < 8 {
                return Err(Box::from("Password is too short"));
            }

            Ok(Self(value.to_string()))
        }
    }

    impl AsRef<str> for Password {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Price(f32);

    impl Price {
        pub fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let value = value
                .parse::<f32>()
                .map_err(|_| "Price must be a number".to_string())?;

            if value <= 0.0 {
                return Err(Box::from("Price must be greater than zero"));
            }

            Ok(Self(value))
        }

        pub fn parse_f32(value: f32) -> Result<Self, Box<dyn std::error::Error>> {
            if value <= 0.0 {
                return Err(Box::from("Price must be greater than zero"));
            }

            Ok(Self(value))
        }

        pub fn as_value(&self) -> f32 {
            self.0
        }
    }

    impl AsRef<f32> for Price {
        fn as_ref(&self) -> &f32 {
            &self.0
        }
    }

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct Value(u32);

    impl Value {
        pub fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let value = value
                .parse::<u32>()
                .map_err(|_| "Value must be a number".to_string())?;

            if value == 0 {
                return Err(Box::from("Value must be greater than zero"));
            }

            Ok(Self(value))
        }

        pub fn parse_i32(value: i32) -> Result<Self, Box<dyn std::error::Error>> {
            if value <= 0 {
                return Err(Box::from("Value must be greater than zero"));
            }

            Ok(Self(value as u32))
        }

        pub fn as_value(&self) -> u32 {
            self.0
        }
    }

    impl AsRef<u32> for Value {
        fn as_ref(&self) -> &u32 {
            &self.0
        }
    }

    #[derive(Clone, Debug)]
    pub struct Product {
        pub column_id: Value,
        pub name: Name,
        pub price: Price,
        pub quantity: Value,
    }

    #[derive(Clone, Debug)]
    pub struct Sale {
        pub date: DateTime<Utc>,
        pub product_name: Name,
        pub price: Price,
    }
}

pub mod interfaces {
    use super::entities::{Price, Product, Sale, Value};
    use async_trait::async_trait;

    #[async_trait]
    pub trait ProductRepository {
        async fn find(&self, column_id: Value) -> Option<Product>;
        async fn save(&mut self, product: Product) -> Result<(), Box<dyn std::error::Error>>;
        async fn find_all(&self) -> Vec<Product>;
    }

    #[async_trait]
    pub trait SaleRepository {
        async fn save(&mut self, sale: Sale) -> Result<(), Box<dyn std::error::Error>>;
        async fn find_all(&self) -> Vec<Sale>;
    }

    pub trait Terminal {
        fn prompt(&self, message: &str) {
            println!("{}", message);
        }
    }

    pub trait PaymentTerminal: Terminal {
        fn request(&self) -> Result<Price, Box<dyn std::error::Error>>;
        fn refund(&self, amount: Price) -> Result<(), Box<dyn std::error::Error>>;
    }
}
