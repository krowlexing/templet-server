use std::io;

use rusqlite::{Error, ErrorCode, Row};

use super::Con;

pub struct Users {
    con: Con,
}

pub struct User {
    id: usize,
    name: String,
    username: String,
    password: String,
}
// select { sql, params_type, row_type}
impl User {
    pub fn from_row(row: &Row) -> Result<User, Error> {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let username = row.get("username")?;
        let password = row.get("password")?;
        Ok(Self {
            id,
            name,
            username,
            password,
        })
    }
}

pub struct NewUser {
    name: String,
    username: String,
    password: String,
}

impl NewUser {
    pub fn new(name: String, username: String, password: String) -> io::Result<NewUser> {
        //TODO: check for validity

        Ok(Self {
            name,
            username,
            password,
        })
    }
}

impl Users {
    pub fn new(con: &Con) -> Self {
        Users { con: con.clone() }
    }

    pub fn create_table(&self) -> Result<usize, Error> {
        let con = self.con.lock().unwrap();

        con.execute(
            "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY,
                    name TEXT,
                    username TEXT UNIQUE,
                    password TEXT
                )",
            (),
        )
    }

    pub fn insert(&self, user: NewUser) -> Result<usize, Error> {
        let hash = bcrypt::hash(user.password, 10).unwrap();
        let con = self.con.lock().unwrap();

        let mut stmt =
            con.prepare_cached("INSERT INTO users(name, username, password) VALUES(?,?,?)")?;
        stmt.execute((user.name, user.username, hash))
    }

    pub fn find_user_by_name(&self, username: &str) -> Result<User, Error> {
        let con = self.con.lock().unwrap();

        let mut stmt = con.prepare_cached("SELECT * FROM users WHERE username = ?")?;

        stmt.query_row([username], User::from_row)
    }

    pub fn register_user(&self, user: &NewUser) -> Result<(), RegistrationError> {
        register_user(self, user)
    }

    pub fn find_user(&self, username: &str, password: &str) -> Result<User, LoginError> {
        find_user(self, username, password)
    }
}

pub enum LoginError {
    UserNotFound,
    WrongPassword,
    SqliteError(Error),
}

impl From<Error> for LoginError {
    fn from(value: Error) -> Self {
        LoginError::SqliteError(value)
    }
}

pub fn find_user(users: &Users, username: &str, password: &str) -> Result<User, LoginError> {
    let con = users.con.lock().unwrap();

    let mut stmt = con.prepare_cached("SELECT * FROM users WHERE username = ?")?;

    let user = stmt.query_row([username], User::from_row).map(|user| {
        bcrypt::verify(password, &user.password)
            .unwrap()
            .then_some(user)
    });

    match user {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(LoginError::WrongPassword),
        Err(Error::QueryReturnedNoRows) => Err(LoginError::UserNotFound),
        Err(e) => Err(LoginError::SqliteError(e)),
    }
}

pub enum RegistrationError {
    UserAlreadyExists,
    SqliteError(Error),
}

impl From<Error> for RegistrationError {
    fn from(value: Error) -> Self {
        RegistrationError::SqliteError(value)
    }
}

fn register_user(users: &Users, user: &NewUser) -> Result<(), RegistrationError> {
    let sql = "INSERT INTO users(name, username, password) VALUES (?, ?, ?)";

    let con = users.con.lock().unwrap();
    let mut stmt = con.prepare_cached(sql)?;
    let result = stmt.execute([&user.name, &user.username, &user.password]);

    match result {
        Ok(_) => Ok(()),
        Err(Error::SqliteFailure(e, o)) => match e.code {
            ErrorCode::ConstraintViolation => Err(RegistrationError::UserAlreadyExists),
            _ => Err(Error::SqliteFailure(e, o))?,
        },
        Err(e) => Err(e)?,
    }
}
