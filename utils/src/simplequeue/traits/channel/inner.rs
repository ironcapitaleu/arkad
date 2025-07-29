/// A trait representing the behavior of an inner channel.
pub trait InnerChannel {
    fn serve(&self);
}

// Example implementation for a real service
pub struct RealService;

impl InnerChannel for RealService {
    fn serve(&self) {
        println!("Real service is working.");
    }
}

// Example implementation for a mock service
pub struct MockService;

impl InnerChannel for MockService {
    fn serve(&self) {
        println!("Mock service for testing.");
    }
}
