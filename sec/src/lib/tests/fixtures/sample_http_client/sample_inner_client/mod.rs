pub mod always_failing;
pub mod always_succeeding;

pub use always_failing::AlwaysFailingHttpClient;
pub use always_succeeding::AlwaysSucceedingHttpClient;
