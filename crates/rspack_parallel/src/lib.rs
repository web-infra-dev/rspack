mod iterator_consumer;
pub mod scope;

pub use iterator_consumer::{FutureConsumer, RayonConsumer, TryFutureConsumer};
pub use scope::{scope, scope_with_limit};
