// SPDX-License-Identifier: GPL-3.0-or-later

use std::process::ExitCode;
use std::time::Duration;

use tokio::signal::unix::{SignalKind, signal};
use winclip_daemon::service::{self, Daemon};
use winclip_daemon::storage::Storage;

const TRIM_INTERVAL: Duration = Duration::from_hours(1);

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("winclip-daemon: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Storage::open(&Storage::default_dir()?)?;
    // zbus lets a newly started daemon take the name over by default, which would leave
    // the old one running without a name but with the database still open.
    let connection = zbus::connection::Builder::session()?
        .serve_at(service::OBJECT_PATH, Daemon::new(storage))?
        .name(service::BUS_NAME)?
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .build()
        .await?;

    let mut terminate = signal(SignalKind::terminate())?;
    let mut trim_timer = tokio::time::interval(TRIM_INTERVAL);
    loop {
        tokio::select! {
            _ = trim_timer.tick() => {
                if let Err(e) = service::trim(&connection).await {
                    eprintln!("winclip-daemon: trimming history failed: {e}");
                }
            }
            _ = tokio::signal::ctrl_c() => break,
            _ = terminate.recv() => break,
            () = connection.closed() => return Err("lost connection to the session bus".into()),
        }
    }
    Ok(())
}
