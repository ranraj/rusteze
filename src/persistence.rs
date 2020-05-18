extern crate rusqlite;

use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

use crate::domain::{Server, DEFAULT_USER};

pub fn init_db(db: &String) -> Result<Response> {
    let conn = Connection::open(db)?;

    conn.execute(
        "create table if not exists user (
             id integer primary key,
             name text not null unique
         )",
        NO_PARAMS,
    )?;
    conn.execute(
        "create table if not exists server (
             id integer primary key,
             title text,
             domain text,
             ip text,
             user_name text not null,
             owner_id integer not null references user(id)
         )",
        NO_PARAMS,
    )?;

    conn.execute(
        "create table if not exists health (             
             name text not null             
         )",
        NO_PARAMS,
    )?;

    Ok(match insert_user(DEFAULT_USER, &conn) {
        Ok(_) => Response::Success,
        Err(_) => {
            //Server : ignore unique constraint error
            //println!("Init {}",e);
            Response::Success
        }
    })
}

pub fn check(conn: &Connection) -> Result<Response> {
    #[derive(Debug)]
    struct Health {
        name: String,
    }

    let name = String::from("health str");
    conn.execute("INSERT INTO health (name) values (?1)", &[&name])?;
    let mut stmt = conn.prepare("SELECT * FROM health;")?;

    let health = stmt.query_map(NO_PARAMS, |row| Ok(Health { name: row.get(0)? }))?;
    conn.execute("DELETE FROM health", NO_PARAMS)?;
    Ok(Response::Success)
}

pub enum CrudAction {
    Save(Server),
    Find(i64),
    Remove(i64),
    FindAll,
    RemoveAll,
    HealthCheck
}
pub enum Response {
    List(Vec<Server>),
    One(Option<Server>),
    Success,
    Error(String),
}

pub fn db_action(action: CrudAction, db: String) -> Response {
    if let Ok(conn) = Connection::open(db) {
        match action {
            CrudAction::Save(server) => insert_server(server, &conn).unwrap(),
            CrudAction::Find(id) => match read_one(id, &conn) {
                Ok(resp) => resp,
                Err(_) => Response::Error("Failure".to_string()),
            },
            CrudAction::FindAll => read_all(&conn).unwrap(),
            CrudAction::Remove(id) => remove_record(id, &conn).unwrap(),
            CrudAction::RemoveAll => remove_all_records(&conn).unwrap(),
            CrudAction::HealthCheck => match check(&conn) {
                Ok(resp) => resp,
                Err(why) => {
                    println!("{}", why);
                    panic!("{}", why)
                }
            },
        }
    } else {
        println!("Db store is not found, Please setup application");
        Response::Error("Db store is not found, Please setup application".to_string())
    }
}

fn insert_user(name: &str, conn: &Connection) -> Result<Response> {
    let last_id: String = conn.last_insert_rowid().to_string();
    conn.execute(
        "INSERT INTO user (id,name) values (?1,?2)",
        &[&last_id, &name.to_string()],
    )?;
    Ok(Response::Success)
}

fn insert_server(server: Server, conn: &Connection) -> Result<Response> {
    conn.execute(
        "INSERT INTO server (title,domain,ip,user_name,owner_id) values (?1,?2,?3,?4,(SELECT id FROM user where name = ?5));",
        &[&server.title.to_string(),&server.domain.unwrap_or("".to_owned()),&server.ip.unwrap_or("".to_owned()),&server.user_name, &DEFAULT_USER.to_string()],
    )?;

    Ok(Response::Success)
}
fn read_one(id: i64, conn: &Connection) -> Result<Response> {
    let mut stmt = conn.prepare(
        "SELECT s.id,s.title,s.domain,s.ip,s.user_name,u.name from server s
        INNER JOIN user u
        ON u.id = s.owner_id where s.id = :id and u.id = (SELECT id FROM user where name = :name)",
    )?;

    let mut rows = stmt.query_named(&[(":id", &id), (":name", &DEFAULT_USER)])?;
    let mut result: Option<Server> = None;
    while let Some(row) = rows.next()? {
        result = Some(Server {
            id: Option::Some(row.get(0)?),
            title: row.get(1)?,
            domain: row.get(2)?,
            ip: row.get(3)?,
            user_name: row.get(4)?,
            owner: row.get(5)?,
        })
    }
    Ok(Response::One(result))
}

fn read_all(conn: &Connection) -> Result<Response> {
    let mut stmt = conn.prepare(
        "SELECT s.id,s.title,s.domain,s.ip,s.user_name,u.name from server s
        INNER JOIN user u
        ON u.id = s.owner_id;",
    )?;
    let servers = stmt.query_map(NO_PARAMS, |row| {
        Ok(Server {
            id: Option::Some(row.get(0)?),
            title: row.get(1)?,
            domain: Some(row.get(2)?),
            ip: Some(row.get(3)?),
            user_name: row.get(4)?,
            owner: row.get(5)?,
        })
    })?;
    let collected: rusqlite::Result<Vec<Server>> = servers.collect();
    let result = match collected {
        Ok(list) => list,
        Err(_) => Vec::<Server>::new(),
    };
    Ok(Response::List(result))
}

fn remove_all_records(conn: &Connection) -> Result<Response> {
    conn.execute("DELETE FROM Server", NO_PARAMS)?;
    Ok(Response::Success)
}
fn remove_record(id: i64, conn: &Connection) -> Result<Response> {
    conn.execute("DELETE FROM Server where id =?", &[&id])?;
    Ok(Response::Success)
}
