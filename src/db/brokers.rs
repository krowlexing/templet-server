use axum_utils::{copy, copy_mut, impl_from_row};
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};

use super::apps::has_permission;

use super::{query_execute, query_rows, Con, SqlResult};

pub struct Brokers {
    con: Con,
}

impl Brokers {
    pub fn new(con: &Con) -> Self {
        Self { con: con.clone() }
    }

    copy!(create_table() -> SqlResult<usize>);
    copy_mut!(for_app(app_id: i32, user_id: i32) -> SqlResult<Vec<Broker>>);
    copy_mut!(create(app_id: i32, user_id: i32, new_broker: NewBroker) -> SqlResult<i32>);
    copy_mut!(delete(app_id: i32, user_id: i32, app_user_id: i32) -> SqlResult<usize>);
}

fn create_table(con: &Connection) -> SqlResult<usize> {
    con.execute(
        "CREATE TABLE IF NOT EXISTS brokers (
            id INTEGER PRIMARY KEY,
            app_id INTEGER,
            name TEXT,
            description TEXT,
            version TEXT,
            active INTEGER,
            stopped INTEGER,
            FOREIGN KEY(app_id) REFERENCES apps(id)
        )",
        [],
    )
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Broker {
    pub id: i32,
    pub app_id: i32,
    pub name: String,
    pub description: String,
    pub version: String,
    pub active: bool,
    pub stopped: bool,
}

impl_from_row!(Broker {
    id,
    app_id,
    name,
    description,
    version,
    active,
    stopped
});

fn for_app(con: &mut Connection, app_id: i32, user_id: i32) -> SqlResult<Vec<Broker>> {
    let tx = con.transaction()?;
    has_permission(&tx, app_id, user_id)?;
    let users = query_rows!(tx => "SELECT * FROM brokers WHERE app_id = ?", [app_id], Broker);
    Ok(users)
}

fn create(
    con: &mut Connection,
    app_id: i32,
    user_id: i32,
    new_broker: NewBroker,
) -> SqlResult<i32> {
    let tx = con.transaction()?;
    has_permission(&tx, app_id, user_id)?;
    let result = unchecked_create(&tx, app_id, new_broker)?;
    tx.commit()?;
    Ok(result)
}

fn delete(con: &mut Connection, app_id: i32, user_id: i32, app_user_id: i32) -> SqlResult<usize> {
    let tx = con.transaction()?;
    has_permission(&tx, app_id, user_id)?;
    let result = unchecked_delete(&tx, app_user_id)?;
    tx.commit()?;
    Ok(result)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBroker {
    name: String,
    description: String,
    stopped: bool,
}

fn unchecked_create(con: &Connection, app_id: i32, new_broker: NewBroker) -> SqlResult<i32> {
    let NewBroker {
        name,
        description,
        stopped,
    } = new_broker;
    query_execute!(con => "INSERT INTO brokers(app_id, name, description, stopped, version, active) VALUES (?, ?, ?, ?, '0.0.0', 0)", (app_id, name, description, stopped))?;
    Ok(con.last_insert_rowid() as i32)
}

fn unchecked_delete(con: &Connection, broker_id: i32) -> SqlResult<usize> {
    query_execute!(con => "DELETE FROM brokers WHERE id = ?", [broker_id])
}
