pub mod always_failing;
pub mod always_responding;
pub mod always_returning;
pub mod always_succeeding;
pub mod routing;

pub use always_failing::AlwaysFailingHttpClient;
pub use always_responding::AlwaysRespondingHttpClient;
pub use always_returning::AlwaysReturningHttpClient;
pub use always_succeeding::AlwaysSucceedingHttpClient;
pub use routing::RoutingHttpClient;
