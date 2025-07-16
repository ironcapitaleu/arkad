use std::path::PathBuf;

use dotenvy;

pub mod constants;

/// Loads environment variables from a `.env` file using `dotenvy`.
///
/// # Errors
///
/// Returns a [`dotenvy::Error`] if the `.env` file is not found, unreadable, or if
/// there's a problem parsing the file's contents.
pub fn load_dotenv_file() -> Result<PathBuf, dotenvy::Error> {
    dotenvy::dotenv()
}

pub fn setup_environment() {
    match load_dotenv_file() {
        Ok(path) => {
            println!(
                "Environment variables loaded successfully from .env file: '{}'",
                path.display()
            );
        }
        Err(e) => eprintln!("Failed to load environment variables from .env file: '{e}'"),
    }
}
