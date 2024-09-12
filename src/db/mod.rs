use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use app_users::AppUsers;
use apps::Apps;
use operators::Operators;
use rusqlite::Connection;
use users::Users;

pub mod app_users;
pub mod apps;
pub mod brokers;
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
    pub app_users: AppUsers,
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
            app_users: AppUsers::new(&con),
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
        self.operators.create_table()?;
        self.app_users.create_table()
    }
}

macro_rules! query_row {
    ($con:ident => $sql:expr, $params:expr, $result:ty) => {{
        let mut stmt = $con.prepare_cached($sql)?;
        stmt.query_row($params, <$result>::from_row)
    }};
}

pub(crate) use query_row;

macro_rules! query_rows {
    ($con:ident => $sql:expr, $params:expr, $result:ty) => {{
        let mut stmt = $con.prepare_cached($sql)?;
        let users = stmt
            .query_map($params, <$result>::from_row)?
            .collect::<Result<Vec<$result>, _>>()
            .unwrap();
        users
    }};
}

pub(crate) use query_rows;

macro_rules! query_execute {
    ($con:ident => $sql:expr, $params:expr) => {{
        $con.execute($sql, $params)
    }};
}

pub(crate) use query_execute;
