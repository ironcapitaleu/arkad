pub trait InnerConnection {
    fn connect(&self) -> Result<(), String>;
}
