mod config;
mod connector;
mod domain;
mod handler;
mod persistence;
mod service;


use clap::{load_yaml, App, ArgMatches};
use handler::{
    handle_add, handle_config_argument, handle_connect, handle_init, handle_list, handle_remove,
    handle_test,
};
/// Command line todo application
/// Below actions can be performed using this application
/// - Init
/// - Test
/// - Add
/// - List by id
/// - List all
/// - Remove all
/// - Remove by id
/// #Example
/// ```
/// xcon init
/// ```
/// The above command initialize the default database and application configuration
///
fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    application(&matches);
}

fn application(matches: &ArgMatches) {
    let settings = handle_config_argument(matches);
    handle_init(matches, &settings);
    handle_test(matches, &settings);
    handle_add(matches, &settings);
    handle_list(matches, &settings);
    handle_remove(matches, &settings);
    handle_connect(matches,&settings);
}
