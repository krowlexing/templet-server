use axum_utils::copy;
use axum_utils::copy_mut;
use axum_utils::impl_from_row;
use rusqlite::Connection;
use rusqlite::Row;
use serde::Deserialize;
use serde::Serialize;

use super::apps::has_permission;
use super::query_execute;
// for_app, create, delete
use super::{query_rows, Con, SqlResult};

pub struct AppUsers {
    con: Con,
}

impl AppUsers {
    pub fn new(con: &Con) -> Self {
        Self { con: con.clone() }
    }

    copy!(create_table() -> SqlResult<usize>);
    copy_mut!(for_app(app_id: i32, user_id: i32) -> SqlResult<Vec<AppUser>>);
    copy_mut!(create(app_id: i32, user_id: i32, new_user_id: i32) -> SqlResult<i32>);
    copy_mut!(delete(app_id: i32, user_id: i32, app_user_id: i32) -> SqlResult<usize>);
}

fn create_table(con: &Connection) -> SqlResult<usize> {
    con.execute(
        "CREATE TABLE IF NOT EXISTS app_users (
        id INTEGER PRIMARY KEY,
        app_id INTEGER,
        user_id INTEGER,
        UNIQUE(app_id, user_id)
    )",
        [],
    )
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUser {
    pub id: i32,
    pub app_id: i32,
    pub user_id: i32,
    pub name: String,
    pub username: String,
}

impl_from_row!(AppUser {
    id,
    app_id,
    user_id,
    name,
    username
});

fn for_app(con: &mut Connection, app_id: i32, user_id: i32) -> SqlResult<Vec<AppUser>> {
    let tx = con.transaction()?;
    has_permission(&tx, app_id, user_id)?;
    let users = query_rows!(tx => "SELECT * FROM app_users JOIN users ON app_users.user_id = users.id WHERE app_id = ?", [app_id], AppUser);
    Ok(users)
}

fn create(con: &mut Connection, app_id: i32, user_id: i32, new_user_id: i32) -> SqlResult<i32> {
    let tx = con.transaction()?;
    has_permission(&tx, app_id, user_id)?;
    let result = unchecked_create(&tx, app_id, new_user_id)?;
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

fn unchecked_create(con: &Connection, app_id: i32, new_user_id: i32) -> SqlResult<i32> {
    query_execute!(con => "INSERT INTO app_users(app_id, user_id) VALUES (?, ?)", [app_id, new_user_id])?;
    Ok(con.last_insert_rowid() as i32)
}

fn unchecked_delete(con: &Connection, app_user_id: i32) -> SqlResult<usize> {
    query_execute!(con => "DELETE FROM app_users WHERE id = ?", [app_user_id])
}
