use crate::terminals::CliTerminal;
use std::error::Error;
use std::fmt::{Display, Formatter};
use vending_machine::application::states::{Admin, Guest, Locked, Supplier, Unlocked};

pub enum PromptPerspective {
    GuestUnlocked(CliTerminal<Guest, Unlocked>),
    GuestLocked(CliTerminal<Guest, Locked>),
    AdminUnlocked(CliTerminal<Admin, Unlocked>),
    AdminLocked(CliTerminal<Admin, Locked>),
    SupplierUnlocked(CliTerminal<Supplier, Unlocked>),
    SupplierLocked(CliTerminal<Supplier, Locked>),
}

impl PromptPerspective {
    pub fn run(self) -> Result<Self, Box<dyn Error>> {
        match self {
            PromptPerspective::GuestUnlocked(terminal) => terminal.run(),
            PromptPerspective::GuestLocked(terminal) => terminal.run(),
            PromptPerspective::AdminUnlocked(terminal) => terminal.run(),
            PromptPerspective::AdminLocked(terminal) => terminal.run(),
            PromptPerspective::SupplierUnlocked(terminal) => terminal.run(),
            PromptPerspective::SupplierLocked(terminal) => terminal.run(),
        }
    }
}

pub(crate) enum GuestUnlockedCommand {
    Login,
    ListProducts,
    BuyProduct,
    Exit,
}

impl Display for GuestUnlockedCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GuestUnlockedCommand::Login => write!(f, "1. Login"),
            GuestUnlockedCommand::ListProducts => write!(f, "2. List Products"),
            GuestUnlockedCommand::BuyProduct => write!(f, "3. Buy Product"),
            GuestUnlockedCommand::Exit => write!(f, "4. Exit"),
        }
    }
}

impl TryFrom<&str> for GuestUnlockedCommand {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(GuestUnlockedCommand::Login),
            "2" => Ok(GuestUnlockedCommand::ListProducts),
            "3" => Ok(GuestUnlockedCommand::BuyProduct),
            "4" => Ok(GuestUnlockedCommand::Exit),
            _ => Err(Box::from("Invalid command")),
        }
    }
}

pub(crate) enum GuestLockedCommand {
    Login,
    ListProducts,
    Exit,
}

impl Display for GuestLockedCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GuestLockedCommand::Login => write!(f, "1. Login"),
            GuestLockedCommand::ListProducts => write!(f, "2. List Products"),
            GuestLockedCommand::Exit => write!(f, "3. Exit"),
        }
    }
}

impl TryFrom<&str> for GuestLockedCommand {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(GuestLockedCommand::Login),
            "2" => Ok(GuestLockedCommand::ListProducts),
            "3" => Ok(GuestLockedCommand::Exit),
            _ => Err(Box::from("Invalid command")),
        }
    }
}

pub(crate) enum AdminUnlockedCommand {
    Logout,
    ListProducts,
    ListSales,
    Lock,
    Exit,
}

impl Display for AdminUnlockedCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminUnlockedCommand::Logout => write!(f, "1. Logout"),
            AdminUnlockedCommand::ListProducts => write!(f, "2. List Products"),
            AdminUnlockedCommand::ListSales => write!(f, "3. List Sales"),
            AdminUnlockedCommand::Lock => write!(f, "4. Lock"),
            AdminUnlockedCommand::Exit => write!(f, "5. Exit"),
        }
    }
}

impl TryFrom<&str> for AdminUnlockedCommand {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(AdminUnlockedCommand::Logout),
            "2" => Ok(AdminUnlockedCommand::ListProducts),
            "3" => Ok(AdminUnlockedCommand::ListSales),
            "4" => Ok(AdminUnlockedCommand::Lock),
            "5" => Ok(AdminUnlockedCommand::Exit),
            _ => Err(Box::from("Invalid command")),
        }
    }
}

pub(crate) enum AdminLockedCommand {
    Logout,
    ListProducts,
    ListSales,
    Unlock,
    Exit,
}

impl Display for AdminLockedCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminLockedCommand::Logout => write!(f, "1. Logout"),
            AdminLockedCommand::ListProducts => write!(f, "2. List Products"),
            AdminLockedCommand::ListSales => write!(f, "3. List Sales"),
            AdminLockedCommand::Unlock => write!(f, "4. Unlock"),
            AdminLockedCommand::Exit => write!(f, "5. Exit"),
        }
    }
}

impl TryFrom<&str> for AdminLockedCommand {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(AdminLockedCommand::Logout),
            "2" => Ok(AdminLockedCommand::ListProducts),
            "3" => Ok(AdminLockedCommand::ListSales),
            "4" => Ok(AdminLockedCommand::Unlock),
            "5" => Ok(AdminLockedCommand::Exit),
            _ => Err(Box::from("Invalid command")),
        }
    }
}

pub(crate) enum SupplierUnlockedCommand {
    Logout,
    ListProducts,
    SupplyProduct,
    Exit,
}

impl Display for SupplierUnlockedCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SupplierUnlockedCommand::Logout => write!(f, "1. Logout"),
            SupplierUnlockedCommand::ListProducts => write!(f, "2. List Products"),
            SupplierUnlockedCommand::SupplyProduct => write!(f, "3. Supply Product"),
            SupplierUnlockedCommand::Exit => write!(f, "4. Exit"),
        }
    }
}

impl TryFrom<&str> for SupplierUnlockedCommand {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(SupplierUnlockedCommand::Logout),
            "2" => Ok(SupplierUnlockedCommand::ListProducts),
            "3" => Ok(SupplierUnlockedCommand::SupplyProduct),
            "4" => Ok(SupplierUnlockedCommand::Exit),
            _ => Err(Box::from("Invalid command")),
        }
    }
}

pub(crate) enum SupplierLockedCommand {
    Logout,
    ListProducts,
    Exit,
}

impl Display for SupplierLockedCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SupplierLockedCommand::Logout => write!(f, "1. Logout"),
            SupplierLockedCommand::ListProducts => write!(f, "2. List Products"),
            SupplierLockedCommand::Exit => write!(f, "3. Exit"),
        }
    }
}

impl TryFrom<&str> for SupplierLockedCommand {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(SupplierLockedCommand::Logout),
            "2" => Ok(SupplierLockedCommand::ListProducts),
            "3" => Ok(SupplierLockedCommand::Exit),
            _ => Err(Box::from("Invalid command")),
        }
    }
}
