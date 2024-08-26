use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use rusqlite::{Connection, Row};
use users::Users;

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
}

impl SqliteDb {
    pub fn new(path: String) -> Result<Self, rusqlite::Error> {
        let con = Con::new(Connection::open(path)
            //.map_err(|_| SqlError::DbFileNotFound)
            ?);

        let db = Self {
            users: Users::new(&con),
            con,
        };

        Ok(db)
    }
}
//     pub fn create_table(&self) -> Result<usize, rusqlite::Error> {
//         self.con.execute(
//             "CREATE TABLE IF NOT EXISTS events (
//                 ordinal INTEGER PRIMARY KEY,
//                 tag INTEGER,
//                 external BOOL,
//                 name TEXT,
//                 data BLOB,
//                 answer BLOB
//             )",
//             (),
//         )
//     }

//     pub fn insert(&self, event: NewEvent) -> Result<Ordinal, rusqlite::Error> {
//         self.con.execute(
//             "INSERT INTO events (tag, external, name, data, answer) VALUES (?1, ?2, ?3, ?4, NULL)",
//             (&event.tag.0, &event.external, &event.name, &event.data)
//         ).unwrap();

//         let id = self.con.last_insert_rowid() as usize;
//         Ok(Ordinal(id))
//     }

//     pub fn answer(&self, ordinal: Ordinal, answer: Vec<u8>) -> Result<(), rusqlite::Error> {
//         self.con
//             .execute(
//                 "UPDATE events SET answer=?1 WHERE ordinal=?2",
//                 (&answer, &ordinal.0),
//             )
//             .unwrap();
//         Ok(())
//     }

//     pub fn read_from(
//         &self,
//         ordinal: Ordinal,
//         allow_external: bool,
//     ) -> Result<Vec<Event>, rusqlite::Error> {
//         let mut stmt = self
//             .con
//             .prepare("SELECT * FROM events WHERE ordinal >= ?1 AND external >= ?2")?;
//         let events = stmt.query_and_then([&ordinal.0, &allow_external.into()], row_to_event)?;

//         let vec: Vec<Event> = events.filter(Result::is_ok).map(Result::unwrap).collect();
//         Ok(vec)
//     }
// }

// fn row_to_event(row: &Row) -> Result<Event, rusqlite::Error> {
//     let ordinal: usize = row.get(0)?;
//     let tag: usize = row.get(1)?;
//     let external: bool = row.get(2)?;
//     let name: String = row.get(3)?;
//     let data: Vec<u8> = row.get(4)?;
//     let answer: Option<Vec<u8>> = row.get(5)?;
//     let answer = answer.map(|vec| serde_json::from_slice::<Answer>(&vec));
//     if let Some(Ok(answer)) = answer {
//         Ok(Event::from_values(
//             Ordinal(ordinal),
//             Tag(tag),
//             external,
//             name,
//             data,
//             Some(answer),
//         ))
//     } else if answer.is_none() {
//         Ok(Event::from_values(
//             Ordinal(ordinal),
//             Tag(tag),
//             external,
//             name,
//             data,
//             None,
//         ))
//     } else {
//         Err(rusqlite::Error::InvalidQuery)
//     }
// }
