use crate::engine::executor::ExecutorRegistry;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub executors: ExecutorRegistry,
}
