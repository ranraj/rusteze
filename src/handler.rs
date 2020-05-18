use cfg_if::cfg_if;
use clap::ArgMatches;
use log::info;
use std::io::{stdin, stdout, Write};

use crate::config::config_router;
use crate::domain::{Action, ActionResponse, Server, Setup};
use crate::service::action_router;

const DELIMETER: &str = "$";

cfg_if! {
    if #[cfg(test)] {
        use crate::domain::MockSettings as Settings;
    } else {
        use crate::domain::Settings;
    }
}

pub fn handle_config_argument(matches: &ArgMatches) -> Settings {
    let base_settings = match Settings::load_config() {
        Ok(settings) => settings,
        Err(why) => {
            info!("Unable to load configuraiton, setting default : {}", why);
            Settings::system_default()
        }
    };
    if matches.is_present("db") {
        let db = matches
            .value_of("db")
            .unwrap_or(base_settings.get_db().as_str())
            .trim()
            .to_lowercase();
        base_settings.update(db)
    } else {
        base_settings
    }
}

pub fn handle_init(matches: &ArgMatches, settings: &Settings) {
    if matches.is_present("init") {
        match config_router(settings, Setup::Init) {
            Ok(_) => println!("Initialization completed successful"),
            Err(why) => println!("Initialization has failed - Reason : {}", why),
        }
    }
}

pub fn handle_test(matches: &ArgMatches, settings: &Settings) {
    if matches.is_present("test") {
        match config_router(settings, Setup::Test) {
            Ok(_) => println!("Test completed successful"),
            Err(_) => println!("Test has failed, Please initalize"),
        }
    }
}

pub fn handle_connect(matches: &ArgMatches, settings: &Settings) {
    if let Some(matches) = matches.subcommand_matches("connect") {
        if let Some(id) = matches.value_of("input").map(|id| id.trim().parse::<i64>()) {
            match id {
                Ok(record_id) => {
                    if let Ok(_) = action_router(settings, Action::Connect(record_id)) {
                        println!("Terminal connected")
                    } else {
                        println!("Connect terminal failed")
                    }
                }
                Err(_) => println!("Save has failed, Please use test command"),
            }
        }
    }
}

pub fn handle_add(matches: &ArgMatches, settings: &Settings) {
    if let Some(_) = matches.subcommand_matches("add") {
        let mut input = read_add_input();
       while let Err(why) = &input {
           println!("{:?}",why);
            retry_prompt("Ip or domain is needed".to_owned());
            input = read_add_input();            
        }
        if let Ok(server) = input {
            match action_router(&settings, Action::Save(server)) {
                Ok(_) => println!("Saved successful"),
                Err(_) => println!("Save has failed, Please use test command"),
            }
        }        
    }
}
pub fn handle_list(matches: &ArgMatches, settings: &Settings) {
    if let Some(matches) = matches.subcommand_matches("list") {
        if let Some(id) = matches.value_of("input").map(|id| id.trim().parse::<i64>()) {
            match id {
                Ok(record_id) => {
                    if let Ok(response) = action_router(&settings, Action::FetchById(record_id)) {
                        match response {
                            ActionResponse::One(server) => {
                                if let Some(record) = server {
                                    let serialized_server = serde_json::to_string(&record).unwrap();
                                    println!("{}", serialized_server);
                                } else {
                                    println!("Record not found")
                                }
                            }
                            _ => println!("Record not found"),
                        }
                    } else {
                        println!("Record not found")
                    }
                }
                Err(_) => println!("Not a valid integer"),
            }
        } else {
            if let Ok(response) = action_router(&settings, Action::Fetch) {
                match response {
                    ActionResponse::All(servers) => {
                        for server in servers {
                            let serialized_server = serde_json::to_string(&server).unwrap();
                            println!("{}", serialized_server);
                        }
                    }
                    _ => println!("Records not found"),
                }
            } else {
                //Server
            }
        }
    }
}

pub fn handle_remove(matches: &ArgMatches, settings: &Settings) {
    if let Some(matches) = matches.subcommand_matches("remove") {
        if let Some(id) = matches.value_of("input").map(|id| id.trim().parse::<i64>()) {
            match id {
                Ok(record_id) => {
                    let message = format!("a record id : {}", record_id);
                    if remove_confirmation(&message) {
                        if let Ok(response) = action_router(settings, Action::DeleteById(record_id))
                        {
                            match response {
                                ActionResponse::Done => {
                                    println!("Successfuly removed a record id {}", record_id)
                                }
                                _ => println!("Record not found"),
                            }
                        } else {
                            //Server
                        }
                    }
                }
                Err(_) => println!("Not a valid integer"),
            }
        } else {
            if remove_confirmation("all records") {
                if let Ok(response) = action_router(settings, Action::Delete) {
                    match response {
                        ActionResponse::Done => println!("Remove all successful "),
                        _ => println!("Record not found"),
                    }
                }
            } else {
                //Server : ignore
            }
        }
    }
}
fn retry_prompt(message: String){
    println!("{} ! Please retry",message);
}
fn read_add_input() -> Result<Server, std::io::Error> {
    let mut title = String::new();
    let mut domain_str = String::new();
    let mut ip_str = String::new();
    let mut user_name = String::new();

    fn get_input(input: &mut String, msg: &str, error: &str) {
        print!("{} {}", msg, DELIMETER);
        let _ = stdout().flush();
        stdin().read_line(input).expect(error);
        clean_input(input);
    }
    let error_message = "Did not enter a correct string";
    get_input(&mut title, "Title", error_message);
    get_input(&mut domain_str, "Domain", error_message);
    get_input(&mut ip_str, "Ip", error_message);
    get_input(&mut user_name, "Username", error_message);
    if domain_str.is_empty() && ip_str.is_empty() {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid Ip or Domain",
        ))
    } else {
        let domain = if domain_str.is_empty() {
            None
        } else {
            Some(domain_str)
        };
        let ip = if ip_str.is_empty() {
            None
        } else {
            Some(ip_str)
        };
        Ok(Server {
            id: Option::None,
            title,
            domain,
            ip,
            user_name,
            owner: Option::None,
        })
    }
}
fn clean_input(s: &mut String) {
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
}

fn remove_confirmation(message: &str) -> bool {
    let mut confirmation = String::new();
    print!(
        "Do you want to remove {} (press enter to continue or type (N/n)) {} ",
        message, DELIMETER
    );
    let _ = stdout().flush();
    stdin()
        .read_line(&mut confirmation)
        .expect("Did not enter a correct string");
    clean_input(&mut confirmation);
    if confirmation.eq_ignore_ascii_case("N") || confirmation.eq_ignore_ascii_case("n") {
        return false;
    } else {
        return true;
    }
}
