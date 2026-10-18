// SPDX-License-Identifier: GPL-3.0-or-later

mod error;
#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use zbus::object_server::SignalEmitter;
use zbus::{Connection, fdo, interface};

use crate::storage::{self, ItemSummary, Storage};
pub use error::Error;

pub const BUS_NAME: &str = "org.winclip.Daemon";
pub const OBJECT_PATH: &str = "/org/winclip/Daemon1";

type Summary = (u64, String, bool, i64);

pub struct Daemon {
    storage: Arc<Mutex<Storage>>,
}

impl Daemon {
    #[must_use]
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
        }
    }

    async fn with_storage<T, F>(&self, work: F) -> Result<T, Error>
    where
        T: Send + 'static,
        F: FnOnce(&mut Storage) -> storage::Result<T> + Send + 'static,
    {
        let storage = Arc::clone(&self.storage);
        let result = tokio::task::spawn_blocking(move || work(&mut lock(&storage)))
            .await
            .map_err(|e| Error::Failed(e.to_string()))?;
        Ok(result?)
    }

    async fn notify_if_removed(removed: usize, emitter: &SignalEmitter<'_>) -> zbus::Result<()> {
        if removed > 0 {
            Self::history_changed(emitter).await?;
        }
        Ok(())
    }
}

#[interface(name = "org.winclip.Daemon1", spawn = false)]
impl Daemon {
    async fn add_item(
        &self,
        text: String,
        html: String,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> Result<u64, Error> {
        let id = self
            .with_storage(move |s| s.add_item(&text, Some(&html)))
            .await?;
        Self::history_changed(&emitter).await?;
        Ok(id)
    }

    async fn get_history(&self, limit: u32) -> Result<Vec<Summary>, Error> {
        let items = self.with_storage(move |s| s.get_history(limit)).await?;
        Ok(to_summaries(items))
    }

    async fn get_item(&self, id: u64) -> Result<(String, String), Error> {
        let item = self.with_storage(move |s| s.get_item(id)).await?;
        Ok((item.text, item.html.unwrap_or_default()))
    }

    async fn delete_item(
        &self,
        id: u64,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> Result<(), Error> {
        self.with_storage(move |s| s.delete_item(id)).await?;
        Self::history_changed(&emitter).await?;
        Ok(())
    }

    async fn clear_history(
        &self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> Result<(), Error> {
        let removed = self.with_storage(Storage::clear_history).await?;
        Self::notify_if_removed(removed, &emitter).await?;
        Ok(())
    }

    async fn search(&self, query: String, limit: u32) -> Result<Vec<Summary>, Error> {
        let items = self.with_storage(move |s| s.search(&query, limit)).await?;
        Ok(to_summaries(items))
    }

    #[zbus(signal)]
    async fn history_changed(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(property)]
    async fn max_items(&self) -> fdo::Result<u32> {
        Ok(self.with_storage(|s| s.max_items()).await?)
    }

    #[zbus(property)]
    async fn set_max_items(
        &self,
        value: u32,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        let removed = self.with_storage(move |s| s.set_max_items(value)).await?;
        Ok(Self::notify_if_removed(removed, &emitter).await?)
    }

    #[zbus(property)]
    async fn max_age_days(&self) -> fdo::Result<u32> {
        Ok(self.with_storage(|s| s.max_age_days()).await?)
    }

    #[zbus(property)]
    async fn set_max_age_days(
        &self,
        value: u32,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        let removed = self
            .with_storage(move |s| s.set_max_age_days(value))
            .await?;
        Ok(Self::notify_if_removed(removed, &emitter).await?)
    }

    #[zbus(property)]
    async fn max_item_bytes(&self) -> fdo::Result<u32> {
        Ok(self.with_storage(|s| s.max_item_bytes()).await?)
    }

    #[zbus(property)]
    async fn set_max_item_bytes(
        &self,
        value: u32,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        let removed = self
            .with_storage(move |s| s.set_max_item_bytes(value))
            .await?;
        Ok(Self::notify_if_removed(removed, &emitter).await?)
    }

    #[zbus(property)]
    #[expect(clippy::unused_self, reason = "zbus properties take &self")]
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}

pub async fn trim(connection: &Connection) -> Result<(), Error> {
    let iface = connection
        .object_server()
        .interface::<_, Daemon>(OBJECT_PATH)
        .await?;
    let removed = iface.get().await.with_storage(Storage::trim).await?;
    Daemon::notify_if_removed(removed, iface.signal_emitter()).await?;
    Ok(())
}

fn lock(storage: &Mutex<Storage>) -> MutexGuard<'_, Storage> {
    storage.lock().unwrap_or_else(PoisonError::into_inner)
}

fn to_summaries(items: Vec<ItemSummary>) -> Vec<Summary> {
    items
        .into_iter()
        .map(|i| (i.id, i.preview, i.has_html, i.copied_at))
        .collect()
}
