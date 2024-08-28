use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use apps::Apps;
use rusqlite::Connection;
use users::Users;

pub mod apps;
pub mod table;
pub mod users;
pub enum SqlError {
    DbFileNotFound,
}

#[derive(Clone)]
pub struct Con(pub Arc<Mutex<Connection>>);

impl Con {
    pub fn new(connection: Connection) -> Self {
        Con(Arc::new(Mutex::new(connection)))
    }
}

impl Deref for Con {
    type Target = Arc<Mutex<Connection>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Db = Arc<SqliteDb>;
pub struct SqliteDb {
    con: Con,
    pub users: Users,
    pub apps: Apps,
}

impl SqliteDb {
    pub fn new(path: String) -> Result<Self, rusqlite::Error> {
        let con = Con::new(Connection::open(path)
            //.map_err(|_| SqlError::DbFileNotFound)
            ?);

        let db = Self {
            users: Users::new(&con),
            apps: Apps::new(&con),
            con,
        };

        Ok(db)
    }

    pub fn init(&self) -> Result<usize, rusqlite::Error> {
        {
            let con = self.con.lock().unwrap();
            con.execute("PRAGMA foreign_keys = ON", ())?;
        }
        self.users.create_table()?;
        self.apps.create_table()
    }
}

#[macro_export]
macro_rules! impl_from_row {
    ($type:ty { $($name:ident),+ }) => {
        impl $type {
            pub fn from_row(row: &Row) -> Result<$type, rusqlite::Error> {
                Ok(Self {
                    $($name: row.get(stringify!($name))?),+
                })
            }
        }
    };
}

pub use impl_from_row;
