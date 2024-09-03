use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use apps::Apps;
use operators::Operators;
use rusqlite::Connection;
use users::Users;

pub mod apps;
pub mod operators;
pub mod table;
pub mod users;

pub type SqlResult<T> = Result<T, rusqlite::Error>;

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
    pub operators: Operators,
}

impl SqliteDb {
    pub fn new(path: String) -> Result<Self, rusqlite::Error> {
        let con = Con::new(Connection::open(path)
            //.map_err(|_| SqlError::DbFileNotFound)
            ?);

        let db = Self {
            users: Users::new(&con),
            apps: Apps::new(&con),
            operators: Operators::new(&con),
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
        self.apps.create_table()?;
        self.operators.create_table()
    }
}
