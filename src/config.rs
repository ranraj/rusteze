
use cfg_if::cfg_if;
use log::info;

use std::fs::File;
use super::domain::{Setup, ActionErr, ActionErrType, ActionResponse};


cfg_if! {
    if #[cfg(test)] {
        use super::domain::MockSettings as Settings;
    } else {
        use super::domain::Settings;
    }
}

pub const CONFIG_FILE: &str = "app.conf";

fn initialize_setup(settings: &Settings) -> Result<ActionResponse, ActionErr> {
    let result_error = initialize_config_file();
    if result_error.is_err() {
        result_error
    } else {
        let write_error = settings.write_default_config();
        if write_error.is_err() {
            write_error
        } else {
            let db_error = settings.initalize_db();
            if db_error.is_err() {
                db_error
            } else {
                Ok(ActionResponse::Done)
            }
        }
    }
}

fn initialize_config_file() -> Result<ActionResponse, ActionErr> {
    if File::open(CONFIG_FILE).is_err() {
        match File::create(CONFIG_FILE) {
            Ok(_) => Ok(ActionResponse::Done),
            _ => Err(ActionErr::build(ActionErrType::UnableToInitialize)),
        }
    } else {
        info!("Config initialized");
        Ok(ActionResponse::Done)
    }
}

pub fn config_router(configuration: &Settings, setup: Setup) -> Result<ActionResponse, ActionErr> {
    match setup {
        Setup::Init => initialize_setup(configuration),
        Setup::Test => {
            if configuration.is_config_available() {
                configuration.test_setup(configuration.get_db())
            } else {
                Err(ActionErr::build(ActionErrType::TestFailed))
            }
        }
    }
}

#[test]
fn initialize_setup_test() {
    let mut mock = Settings::new();
    mock.expect_write_default_config()
        .returning(|| Ok(ActionResponse::Done));
    mock.expect_initalize_db()
        .returning(|| Ok(ActionResponse::Done));
    let response = initialize_setup(&mock);
    assert!(matches!(response, Ok(ActionResponse::Done)));
}

#[test]
fn load_config_test() {
    let settings_ctx = Settings::load_config_context();

    let _settings = settings_ctx.expect().returning(|| Ok(Settings::new()));
    let response = Settings::load_config();
    assert!(matches!(response, _settings));
}
