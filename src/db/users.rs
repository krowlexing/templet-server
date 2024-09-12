use std::io;

use axum_utils::impl_from_row;
use rusqlite::{Connection, Error, ErrorCode, Row};
use serde::{Deserialize, Serialize};

use super::{Con, SqlResult};

pub struct Users {
    con: Con,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub password: String,
}

impl_from_row!(User {
    id,
    name,
    username,
    password
});

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

    pub fn insert(&self, user: NewUser) -> Result<i32, Error> {
        let hash = bcrypt::hash(user.password, 10).unwrap();
        let con = self.con.lock().unwrap();

        let mut stmt =
            con.prepare_cached("INSERT INTO users(name, username, password) VALUES(?,?,?)")?;
        stmt.execute((user.name, user.username, hash))?;
        Ok(con.last_insert_rowid() as i32)
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

    pub fn search(&self, query: &str) -> Result<Vec<UserView>, Error> {
        let con = self.con.lock().unwrap();
        search(&con, query)
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

#[derive(Serialize, Deserialize)]
pub struct UserView {
    pub id: usize,
    pub name: String,
    pub username: String,
}

impl_from_row!(UserView { id, name, username });

fn search(con: &Connection, query: &str) -> Result<Vec<UserView>, rusqlite::Error> {
    let mut stmt =
        con.prepare_cached("SELECT * FROM users WHERE users.username LIKE '%' || ? || '%'")?;
    let users = stmt
        .query_map([query], UserView::from_row)?
        .collect::<Result<_, _>>()
        .unwrap();
    Ok(users)
}

pub fn user_exists(con: &Connection, user_id: i32) -> SqlResult<()> {
    let mut stmt = con.prepare_cached("SELECT * FROM users WHERE id = ?")?;
    stmt.query_row([user_id], |_| Ok(()))
}
