use clap::{App, Arg, ArgMatches};
use std::ops::Deref;

const CONFIGS: &str = "configs";
const WS_EMULATE: &str = "ws_emulate";
const REPOSITORY: &str = "repository";

pub struct Args(ArgMatches);

impl Args {
    pub fn new() -> Self {
        let am = App::new("incoming price manager")
            .version("0.1.0")
            .about("tinkoff investments microservice for price stream reading")
            .arg(
                Arg::new(CONFIGS)
                    .short('c')
                    .long(CONFIGS)
                    .value_name("PATH TO CONFIGS")
                    .about("sets a custom path to configuration files"),
            )
            .arg(
                Arg::new(WS_EMULATE)
                    .short('e')
                    .long(WS_EMULATE)
                    .value_name("WS EMULATE")
                    .takes_value(false)
                    .about("sets a ws emulate mode"),
            )
            .arg(
                Arg::new(REPOSITORY)
                    .short('r')
                    .long(REPOSITORY)
                    .value_name("REPOSITORY SENDING")
                    .takes_value(false)
                    .about("sets a to repository sending mode"),
            )
            .get_matches();

        Args(am)
    }

    pub fn get_configs_path(&self) -> Option<&str> {
        self.value_of(CONFIGS)
    }

    pub fn is_ws_emulate(&self) -> bool {
        self.is_present(WS_EMULATE)
    }

    pub fn is_repository(&self) -> bool {
        self.is_present(REPOSITORY)
    }
}

impl Deref for Args {
    type Target = ArgMatches;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
