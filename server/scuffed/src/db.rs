use tokio::sync::RwLock;

use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Connection(Arc<RwLock<rusqlite::Connection>>);

impl Connection {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;

        Ok(Connection(Arc::new(RwLock::new(conn))))
    }

    pub async fn read<T, F: FnOnce(&rusqlite::Connection) -> T>(&self, func: F) -> T {
        let conn = self.0.read().await;

        func(&*conn)
    }

    pub async fn write<T, F: FnOnce(&mut rusqlite::Connection) -> T>(&self, func: F) -> T {
        let mut conn = self.0.write().await;

        func(&mut *conn)
    }
}
