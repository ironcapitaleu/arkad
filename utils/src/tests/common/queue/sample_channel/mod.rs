use crate::simplequeue::traits::InnerChannel;

#[derive(Debug, Clone)]
pub struct FakeChannel;

impl InnerChannel for FakeChannel {
    fn serve(&self) {
        println!("Fake channel is serving.");
    }
}
