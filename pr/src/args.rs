use anyhow::{anyhow, Context};
use clap::{App, Arg, ArgMatches};
use std::ops::Deref;

const CONFIGS: &str = "configs";
const MIGRATIONS: &str = "migrations";
const STORING: &str = "storing";
const READING: &str = "reading";
const SPEED: &str = "speed";

pub struct Args(ArgMatches);

pub enum Mode {
    Storing,
    Reading { date: String, speed: u16 },
}

impl Args {
    pub fn new() -> Self {
        let am = App::new("price repository")
            .version("0.1.0")
            .about("tinkoff investments microservice for storage")
            .arg(
                Arg::new(CONFIGS)
                    .short('c')
                    .long(CONFIGS)
                    .value_name("PATH TO CONFIGS")
                    .about("sets a custom path to configuration files"),
            )
            .arg(
                Arg::new(MIGRATIONS)
                    .short('m')
                    .long(MIGRATIONS)
                    .value_name("PATH TO MIGRATIONS")
                    .about("sets a custom path to migrations files"),
            )
            .arg(
                Arg::new(STORING)
                    .short('s')
                    .long(STORING)
                    .value_name("STORING MODE")
                    .takes_value(false)
                    .conflicts_with_all(&[READING, SPEED])
                    .about("sets a storing mode"),
            )
            .arg(
                Arg::new(READING)
                    .short('r')
                    .long(READING)
                    .value_name("READING MODE")
                    .conflicts_with(STORING)
                    .requires(SPEED)
                    .about("sets a reading mode"),
            )
            .arg(
                Arg::new(SPEED)
                    .index(1)
                    .default_value("1")
                    .about("sets a speed rate reading"),
            )
            .get_matches();

        Args(am)
    }

    pub fn get_configs_path(&self) -> &str {
        self.value_of(CONFIGS).unwrap_or("")
    }

    pub fn get_migrations_path(&self) -> &str {
        self.value_of(MIGRATIONS).unwrap_or("")
    }

    pub fn get_mode(&self) -> anyhow::Result<Mode> {
        if self.is_present(STORING) {
            return Ok(Mode::Storing);
        }

        // В случае режима чтения мы указываем не только флаг, но и дату, для которой запускаем этот режим
        let date = self
            .value_of(READING)
            .context("startup mode not defined, must be specified -s or -r (storing or reading)")?
            .to_string();

        let speed = self.get_speed()?;

        Ok(Mode::Reading { date, speed })
    }

    fn get_speed(&self) -> anyhow::Result<u16> {
        let speed = self.value_of(SPEED).unwrap_or("1");
        let result = speed
            .parse::<u16>()
            .context("speed must by only unsigned integer: 1, 2, 3, ...")?;

        if result == 0 {
            return Err(anyhow!("speed must be greater than zero"));
        }

        Ok(result)
    }
}

// Deref тут особого смысла не имеет, т.к. есть врапперы: get_configs_path и др., но пусть будет
impl Deref for Args {
    type Target = ArgMatches;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
