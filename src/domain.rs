use crate::config::CONFIG_FILE;
use crate::persistence::{db_action, init_db, CrudAction, Response};
use log::{info, warn};
use mockall::*;
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;

const DEFAULT_DB_NAME: &str = "Server";
pub const DEFAULT_USER: &str = "root";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Server {
    pub id: Option<i32>,
    pub title: String,
    pub domain: Option<String>,
    pub ip: Option<String>,
    pub user_name : String,
    pub owner: Option<String>,
}
impl Server {
    pub fn new(title: String, content: String,domain: Option<String>,ip: Option<String>,user_name: String) -> Self {
        Self {
            id: None,
            title,
            domain,
            ip,
            user_name,
            owner: Some(DEFAULT_USER.to_owned()),
        }
    }
}
pub type ID = i64;
pub enum Action {
    Save(Server),
    Fetch, //Server : Pagination
    FetchById(ID),
    Delete,
    DeleteById(ID),
    Connect(ID)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ActionResponse {
    Done,
    One(Option<Server>),
    All(Vec<Server>),
    Empty,
}

#[derive(Debug)]
pub struct ActionErr {
    msg: String,
    error_type: ActionErrType,
}
#[derive(Debug)]
pub enum ActionErrType {
    InitNotAvailable,
    UnableToInitialize,
    TestFailed,
    RecordNotFound,
    ActionFailed(String)
}
impl ActionErr {
    pub fn build(server: ActionErrType) -> ActionErr {
        match server {
            ActionErrType::InitNotAvailable => ActionErr {
                msg: "Please initialize application,use help".to_owned(),
                error_type: ActionErrType::InitNotAvailable,
            },
            ActionErrType::UnableToInitialize => ActionErr {
                msg: "Unable to initizlize application, contact support".to_owned(),
                error_type: ActionErrType::UnableToInitialize,
            },
            ActionErrType::TestFailed => ActionErr {
                msg: "Database check has failed".to_owned(),
                error_type: ActionErrType::UnableToInitialize,
            },
            ActionErrType::RecordNotFound => ActionErr {
                msg: "Record Not Found".to_owned(),
                error_type: ActionErrType::RecordNotFound,
            },
            ActionErrType::ActionFailed(message) => ActionErr {
                msg: message.to_owned(),
                error_type: ActionErrType::RecordNotFound,
            },
        }
    }
}
impl fmt::Display for ActionErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for ActionErr {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub enum Setup {
    Init,
    Test,
}
struct ConfigurationArgument {
    db: bool,
    set: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Settings {
    pub db: String,
    pub is_saved: bool,
}

#[automock]
impl Settings {
    pub fn system_default() -> Self {
        Self {
            db: DEFAULT_DB_NAME.to_string(),
            is_saved: false,
        }
    }
    pub fn create(db: String, is_saved: bool) -> Self {
        Self { db, is_saved }
    }

    pub fn update(&self, db: String) -> Self {
        Self {
            db,
            is_saved: self.is_saved,
        }
    }
    pub fn get_db(&self) -> String {
        format!("{}.store", self.db.to_owned())
    }
    pub fn test_setup(&self, db: String) -> Result<ActionResponse, ActionErr> {
        match db_action(CrudAction::HealthCheck, db) {
            Response::Success => Ok(ActionResponse::Done),
            _ => Err(ActionErr::build(ActionErrType::InitNotAvailable)),
        }
    }
    pub fn write_default_config(&self) -> Result<ActionResponse, ActionErr> {
        let file_options = OpenOptions::new().write(true).open(CONFIG_FILE);

        match file_options {
            Ok(mut file) => match file.write(self.to_string().as_bytes()) {
                Ok(_) => Ok(ActionResponse::Done),
                Err(why) => {
                    info!("couldn't write to {}", why);
                    Err(ActionErr::build(ActionErrType::UnableToInitialize))
                }
            },
            Err(why) => {
                info!("couldn't write to {}", why);
                Err(ActionErr::build(ActionErrType::UnableToInitialize))
            }
        }
    }
    pub fn write_custom_config(&self) -> Result<ActionResponse, ActionErr> {
        Ok(ActionResponse::Done)
    }
    //Server : Improve with Option<Configuration> for load_config()
    pub fn is_config_available(&self) -> bool {
        self.is_saved
    }
    pub fn initalize_db(&self) -> Result<ActionResponse, ActionErr> {
        let db = format!("{}.store", self.db);
        match init_db(&db) {
            Ok(_) => Ok(ActionResponse::Done),
            Err(why) => {
                warn!("Unable to initiazlize the DB {}", why);
                Err(ActionErr::build(ActionErrType::UnableToInitialize))
            }
        }
    }

    pub fn load_config() -> Result<Self, ActionErr> {
        match File::open(CONFIG_FILE) {
            Ok(config_file) => {
                let buf_reader = BufReader::new(config_file);
                let mut db: String = DEFAULT_DB_NAME.to_owned();

                for (_, line) in buf_reader.lines().enumerate() {
                    let line = line.unwrap();
                    let split = line.split("=");
                    let vec = split.collect::<Vec<&str>>();

                    match vec[0] {
                        "db" => db = vec[1].trim().to_string(),
                        _ => (),
                    }
                }
                Ok(Settings::create(db, true))
            }
            Err(_) => Err(ActionErr::build(ActionErrType::InitNotAvailable)),
        }
    }
}
impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "db={} \n", &self.db.trim().replace(".store", ""))
    }
}
