// SPDX-License-Identifier: GPL-3.0-or-later

mod errors;
mod methods;
mod properties;
mod signals;

use std::time::Duration;

use futures_util::{Stream, StreamExt};
use tokio::net::UnixStream;
use zbus::proxy::CacheProperties;
use zbus::{Connection, Guid, connection, proxy};

use super::{Daemon, OBJECT_PATH};
use crate::storage::Storage;

#[proxy(
    interface = "org.winclip.Daemon1",
    default_service = "org.winclip.Daemon",
    default_path = "/org/winclip/Daemon1"
)]
trait Daemon1 {
    fn add_item(&self, text: &str, html: &str) -> zbus::Result<u64>;
    fn get_history(&self, limit: u32) -> zbus::Result<Vec<(u64, String, bool, i64)>>;
    fn get_item(&self, id: u64) -> zbus::Result<(String, String)>;
    fn delete_item(&self, id: u64) -> zbus::Result<()>;
    fn clear_history(&self) -> zbus::Result<()>;
    fn search(&self, query: &str, limit: u32) -> zbus::Result<Vec<(u64, String, bool, i64)>>;

    #[zbus(signal)]
    fn history_changed(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn max_items(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn set_max_items(&self, value: u32) -> zbus::Result<()>;
    #[zbus(property)]
    fn max_age_days(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn set_max_age_days(&self, value: u32) -> zbus::Result<()>;
    #[zbus(property)]
    fn max_item_bytes(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn set_max_item_bytes(&self, value: u32) -> zbus::Result<()>;
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}

async fn connect() -> (Connection, Daemon1Proxy<'static>) {
    let (server_end, client_end) = UnixStream::pair().unwrap();
    let daemon = Daemon::new(Storage::open_in_memory().unwrap());
    let (server, client) = tokio::try_join!(
        connection::Builder::unix_stream(server_end)
            .server(Guid::generate())
            .unwrap()
            .p2p()
            .serve_at(OBJECT_PATH, daemon)
            .unwrap()
            .build(),
        connection::Builder::unix_stream(client_end).p2p().build(),
    )
    .unwrap();
    let proxy = Daemon1Proxy::builder(&client)
        .cache_properties(CacheProperties::No)
        .build()
        .await
        .unwrap();
    (server, proxy)
}

fn error_name(e: &zbus::Error) -> String {
    match e {
        zbus::Error::MethodError(name, _, _) => name.to_string(),
        zbus::Error::FDO(e) => zbus::DBusError::name(e.as_ref()).to_string(),
        other => panic!("not a D-Bus error reply: {other}"),
    }
}

async fn next_signal<S: Stream + Unpin>(signals: &mut S) {
    tokio::time::timeout(Duration::from_secs(5), signals.next())
        .await
        .expect("no HistoryChanged signal within 5 seconds")
        .expect("signal stream ended");
}
