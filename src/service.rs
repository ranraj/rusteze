use crate::domain::{Action, Server, ActionErr, ActionErrType, ActionResponse, ID};
use crate::persistence::{db_action, CrudAction, Response};
use cfg_if::*;
use crate::connector::connect_terminal;
use log::info;

cfg_if! {
    if #[cfg(test)] {
        use super::domain::MockSettings as Settings;
    } else {
        use super::domain::Settings;
    }
}

pub fn action_router(configuration: &Settings, action: Action) -> Result<ActionResponse, ActionErr> {
    let db = configuration.get_db();
    if configuration.is_config_available() {
        match action {
            Action::Save(server) => save(server, db),
            Action::Fetch => fetch(db),
            Action::FetchById(id) => fetch_by_id(id, db),
            Action::Delete => delete(db),
            Action::DeleteById(id) => delete_by_id(id, db),
            Action::Connect(id) => connect(id,db)
        }
    } else {
        Err(ActionErr::build(ActionErrType::InitNotAvailable))
    }
}
fn connect(id: ID, db: String) -> Result<ActionResponse,ActionErr> {
    if let Ok(response) = fetch_by_id(id,db){
        let error = Err(ActionErr::build(ActionErrType::ActionFailed("Connect Action failed".to_owned()))); 
        match response {
            ActionResponse::One(server_opt) => {              
              if let Some(server) = server_opt{
                  if server.domain.is_some() {
                     match connect_terminal(server.user_name, server.domain.unwrap()){
                        Ok(_) => Ok(ActionResponse::Done),
                        Err(why) =>  {
                            info!("Unable to connect {}",why);
                            error
                        }
                     }                       
                  }else if server.ip.is_some(){
                    match connect_terminal(server.user_name, server.ip.unwrap()){
                        Ok(_) => Ok(ActionResponse::Done),
                        Err(why) =>  {
                            info!("Unable to connect {}",why);
                            error
                        }
                    }                     
                  }else {
                    error
                  }                
              }else{
                error
              }              
            },
            _ => error
        }
    }else{
        Err(ActionErr::build(ActionErrType::InitNotAvailable)) 
    }
    
}
fn save(server: Server, db: String) -> Result<ActionResponse, ActionErr> {
    match db_action(CrudAction::Save(server), db) {
        Response::Success => Ok(ActionResponse::Done),
        _ => Err(ActionErr::build(ActionErrType::ActionFailed("Save action failed".to_owned())))
    }
    
}
fn fetch(db: String) -> Result<ActionResponse, ActionErr> {
    Ok(match db_action(CrudAction::FindAll, db) {
        Response::List(result) => {
            if result.is_empty(){
                ActionResponse::Empty
            }else{
                ActionResponse::All(result)
            }
        }
        _ => ActionResponse::Empty,
    })
}
fn fetch_by_id(id: ID, db: String) -> Result<ActionResponse, ActionErr> {
    Ok(match db_action(CrudAction::Find(id), db) {
        Response::One(result) => ActionResponse::One(result),
        _ => ActionResponse::Empty,
    })
}

fn delete(db: String) -> Result<ActionResponse, ActionErr> {
    match db_action(CrudAction::RemoveAll, db) {
        Response::Success => Ok(ActionResponse::Done),
        _ => Err(ActionErr::build(ActionErrType::InitNotAvailable)),
    }
}

fn delete_by_id(id: ID, db: String) -> Result<ActionResponse, ActionErr> {
    match db_action(CrudAction::Remove(id), db) {
        Response::Success => Ok(ActionResponse::Done),
        _ => Err(ActionErr::build(ActionErrType::InitNotAvailable)),
    }
}
