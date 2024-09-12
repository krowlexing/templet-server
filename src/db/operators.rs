use axum_utils::{copy, impl_from_row};
use serde::{Deserialize, Serialize};

use crate::db::apps::has_permission;

use super::{users::user_exists, Con, SqlResult};
use rusqlite::{Connection, Row};

pub struct Operators {
    con: Con,
}

impl Operators {
    pub fn new(con: &Con) -> Self {
        Self { con: con.clone() }
    }

    copy!(create_table() -> SqlResult<usize>);
    copy!(create(app_id: i32, owner_id: i32, new_operator_id: i32) -> SqlResult<Operator>);
    copy!(delete(app_id: i32, user_id: i32, operator_id: i32) -> SqlResult<usize>);
    copy!(for_app(app_id: i32, user_id: i32) -> SqlResult<Vec<Operator>>);
}

fn create_table(con: &Connection) -> SqlResult<usize> {
    con.execute(
        "CREATE TABLE IF NOT EXISTS operators(
        id INTEGER PRIMARY KEY,
        app_id INTEGER,
        user_id INTEGER,
        FOREIGN KEY (app_id) REFERENCES apps(id),
        FOREIGN KEY (user_id) REFERENCES users(id),
        UNIQUE(app_id, user_id)
    )",
        [],
    )
}

// TODO: custom error type (sql error / permission denied / app not found)
fn create(
    con: &Connection,
    app_id: i32,
    owner_id: i32,
    new_operator_id: i32,
) -> SqlResult<Operator> {
    has_permission(con, app_id, owner_id).map_err(|e| {
        println!(
            "no permission for user {owner_id} to add operator {new_operator_id} to app {app_id}"
        );
        e
    })?;
    user_exists(con, new_operator_id).inspect_err(|_| println!("user doesnt exist"))?;

    let mut stmt = con.prepare_cached("INSERT INTO operators (app_id, user_id) VALUES(?,?)")?;
    stmt.execute([app_id, new_operator_id])?;
    let inserted_id = con.last_insert_rowid();

    con.query_row(
        "SELECT * FROM operators 
        JOIN users ON users.id = user_id
        WHERE operators.id = ?",
        [inserted_id],
        Operator::from_row,
    )
}

// TODO: custom error type
// maybe app id is not needed
fn delete(con: &Connection, app_id: i32, user_id: i32, operator_id: i32) -> SqlResult<usize> {
    let mut stmt = con.prepare_cached(
        "
    SELECT * FROM apps 
    JOIN operators ON operators.app_id = apps.id 
    JOIN users ON operators.user_id = users.id 
    WHERE operators.id = ? AND apps.author_id = ? AND apps.id = ?",
    )?;
    stmt.query_row([operator_id, user_id, app_id], |_| Ok(()))?;

    let mut stmt = con.prepare_cached("DELETE FROM operators WHERE id = ?")?;
    stmt.execute([operator_id])
}

fn for_app(con: &Connection, app_id: i32, user_id: i32) -> SqlResult<Vec<Operator>> {
    has_permission(con, app_id, user_id).map_err(|e| {
        println!("user doesnt have permission");
        e
    })?;

    println!("user has permission");
    let mut stmt = con.prepare_cached(
        "
    SELECT operators.id as id, * FROM operators 
    JOIN users ON user_id = users.id 
    JOIN apps ON operators.app_id = apps.id
    WHERE app_id = ? AND apps.author_id = ?",
    )?;
    let x = stmt
        .query_map([app_id, user_id], Operator::from_row)?
        .collect::<Result<_, _>>();
    x
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    id: i32,
    app_id: i32,
    user_id: i32,
    name: String,
    username: String,
}

impl_from_row!(Operator {
    id,
    app_id,
    user_id,
    name,
    username
});
