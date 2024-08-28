use std::mem::transmute;

use rusqlite::Row;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::Con;

pub struct Apps {
    con: Con,
}

#[derive(Serialize, Deserialize)]
pub struct NewApp {
    pub author: String,
    pub title: String,
    pub description: String,
    pub weblink: String,
    pub version: String,
    pub public: bool,
    pub status: AppStatus,
}

#[repr(usize)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
pub enum AppStatus {
    Active = 0,
    Passive,
    Stopped,
    Blocked,
}

impl From<usize> for AppStatus {
    fn from(value: usize) -> Self {
        unsafe { transmute(value) }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppEntity {
    pub id: usize,
    pub author: usize,
    pub title: String,
    pub description: String,
    pub weblink: String,
    pub version: String,
    pub public: bool,
    pub status: AppStatus,
}

impl AppEntity {
    pub fn from_row(row: &Row) -> Result<AppEntity, rusqlite::Error> {
        let status: usize = row.get(7)?;
        Ok(Self {
            id: row.get(0)?,
            author: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            weblink: row.get(4)?,
            version: row.get(5)?,
            public: row.get(6)?,
            status: status.into(),
        })
    }
}

impl Apps {
    pub fn new(con: &Con) -> Self {
        Self { con: con.clone() }
    }

    pub fn create_table(&self) -> Result<usize, rusqlite::Error> {
        let con = self.con.lock().unwrap();
        con.execute(
            "CREATE TABLE IF NOT EXISTS apps (
            id INTEGER PRIMARY KEY,
            author_id INTEGER,
            title TEXT,
            description TEXT,
            weblink TEXT,
            version TEXT,
            public INTEGER,
            status INTEGER,
            FOREIGN KEY (author_id) REFERENCES users(id)
        )",
            (),
        )
    }

    /**
     possible errors:
      - author does not exist
    */
    pub fn insert(
        &self,
        NewApp {
            author,
            title,
            description,
            weblink,
            version,
            public,
            status,
        }: NewApp,
    ) -> Result<usize, rusqlite::Error> {
        let con = self.con.lock().unwrap();

        con.execute(
            "INSERT INTO apps(
                author_id,
                title, 
                description,
                weblink,
                version,
                public,
                status
            ) SELECT
            id,?,?,?,?,?,? FROM users WHERE users.name = ?",
            (
                title,
                description,
                weblink,
                version,
                public,
                status as usize,
                author,
            ),
        )
    }

    pub fn select_all(&self) -> Result<Vec<AppEntity>, rusqlite::Error> {
        let con = self.con.lock().unwrap();

        let mut stmt = con.prepare_cached("SELECT * FROM apps")?;
        let apps: Vec<AppEntity> = stmt
            .query(())?
            .mapped(AppEntity::from_row)
            .collect::<Result<_, _>>()
            .unwrap();

        Ok(apps)
    }

    pub fn search_by_name(&self, app_title: String) -> Result<Vec<AppEntity>, rusqlite::Error> {
        let con = self.con.lock().unwrap();

        let mut stmt =
            con.prepare_cached("SELECT * FROM apps WHERE apps.title LIKE '%' || ? || '%'")?;

        let apps = stmt
            .query([app_title])?
            .mapped(AppEntity::from_row)
            .collect::<Result<_, _>>()
            .unwrap();

        Ok(apps)
    }
}
