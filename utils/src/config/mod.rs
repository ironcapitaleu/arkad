use std::path::PathBuf;

use dotenvy;

pub mod constants;

pub fn load_dotenv_file() -> Result<PathBuf, dotenvy::Error> {
    let result = dotenvy::dotenv();

    result
}

pub fn setup_environment() -> () {
    match load_dotenv_file() {
        Ok(path) => {
            println!("Environment variables loaded successfully from .env file: '{path:?}'")
        }
        Err(e) => eprintln!("Failed to load environment variables from .env file: '{e}'"),
    }
}
