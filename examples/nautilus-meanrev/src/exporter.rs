//! Streams fills out of the node to another thread, e.g. for a database or a dashboard.
//!
//! Handlers run on the engine thread, so they must never block it: records go
//! into a bounded channel with `try_send`, and are dropped (and counted) if the
//! consumer falls behind. Writing happens on a separate thread.

use std::{
    cell::Cell,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    rc::Rc,
    sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    thread::JoinHandle,
};

use nautilus_common::{
    actor::{DataActor, DataActorConfig, DataActorCore},
    msgbus::{self, TypedHandler},
    nautilus_actor,
};
use nautilus_model::{
    events::{OrderEventAny, OrderFilled},
    identifiers::ActorId,
};
use serde::Serialize;

/// One fill, flattened for export.
#[derive(Clone, Debug, Serialize)]
pub struct FillRecord {
    pub ts_event: u64,
    pub strategy_id: String,
    pub instrument_id: String,
    pub client_order_id: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub commission: Option<String>,
}

impl From<&OrderFilled> for FillRecord {
    fn from(fill: &OrderFilled) -> Self {
        Self {
            ts_event: fill.ts_event.as_u64(),
            strategy_id: fill.strategy_id.to_string(),
            instrument_id: fill.instrument_id.to_string(),
            client_order_id: fill.client_order_id.to_string(),
            side: format!("{:?}", fill.order_side),
            quantity: fill.last_qty.as_f64(),
            price: fill.last_px.as_f64(),
            commission: fill.commission.map(|c| c.to_string()),
        }
    }
}

/// Order events for every strategy are published under `events.order.{strategy_id}`.
const ORDER_EVENTS: &str = "events.order.*";

/// Forwards every fill in the node to `sink`.
pub struct FillExporter {
    core: DataActorCore,
    sink: SyncSender<FillRecord>,
    handler: Option<TypedHandler<OrderEventAny>>,
    dropped: Rc<Cell<u64>>,
}

impl FillExporter {
    pub fn new(actor_id: &str, sink: SyncSender<FillRecord>) -> Self {
        Self {
            core: DataActorCore::new(DataActorConfig {
                actor_id: Some(ActorId::from(actor_id)),
                ..Default::default()
            }),
            sink,
            handler: None,
            dropped: Rc::new(Cell::new(0)),
        }
    }
}

nautilus_actor!(FillExporter);

impl std::fmt::Debug for FillExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FillExporter").finish()
    }
}

impl DataActor for FillExporter {
    fn on_start(&mut self) -> anyhow::Result<()> {
        let sink = self.sink.clone();
        let dropped = self.dropped.clone();
        // DataActor has no order callbacks, so subscribe on the message bus directly.
        let handler = TypedHandler::from(move |event: &OrderEventAny| {
            if let OrderEventAny::Filled(fill) = event {
                match sink.try_send(FillRecord::from(fill)) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => dropped.set(dropped.get() + 1),
                    Err(TrySendError::Disconnected(_)) => {}
                }
            }
        });
        msgbus::subscribe_order_events(ORDER_EVENTS.into(), handler.clone(), None);
        self.handler = Some(handler);
        Ok(())
    }

    fn on_stop(&mut self) -> anyhow::Result<()> {
        // Raw message-bus subscriptions are not tracked by the actor core.
        if let Some(handler) = self.handler.take() {
            msgbus::unsubscribe_order_events(ORDER_EVENTS.into(), &handler);
        }
        if self.dropped.get() > 0 {
            log::warn!(
                "Dropped {} fill records: consumer too slow",
                self.dropped.get()
            );
        }
        Ok(())
    }
}

/// A bounded channel plus a thread that writes each record as one JSON line.
///
/// The thread ends when every sender is dropped and returns the number of lines
/// written. The exporter holds a sender until the engine or node is disposed.
pub fn spawn_jsonl_writer(
    path: &Path,
    capacity: usize,
) -> anyhow::Result<(SyncSender<FillRecord>, JoinHandle<anyhow::Result<usize>>)> {
    let mut out = BufWriter::new(File::create(path)?);
    let (sender, receiver): (SyncSender<FillRecord>, Receiver<FillRecord>) = sync_channel(capacity);
    let handle = std::thread::spawn(move || {
        let mut lines = 0;
        for record in receiver {
            serde_json::to_writer(&mut out, &record)?;
            out.write_all(b"\n")?;
            lines += 1;
        }
        out.flush()?;
        Ok(lines)
    });
    Ok((sender, handle))
}
