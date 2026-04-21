use std::any::TypeId;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    RwLock,
};
use std::time::Instant;
use once_cell::sync::Lazy;

use crate::{
    channel::*,
    error::*,
    message::*,
};

static MESSAGE_BUS: Lazy<MessageBus> = Lazy::new(MessageBus::default);

/// Send a message on the global message bus
///
/// * msg: M where M: Message - The message to send
///
/// -> Result<u64> - the ID of the sent message
pub fn send<M: Message>(msg: M) -> Result<u64> {
    MESSAGE_BUS.send(msg)
}

/// Register a handler with an optional message type filter
///
/// * handler: F where F: FnMut(&M) + Send + Sync + 'static - the handler function
/// * msg_type: Option<std::any::TypeId> - optional message type filter
///
/// -> Result<()> - Ok if the handler was registered successfully
pub fn register_handler(handler: Box<dyn MessageHandler>, msg_type: Option<TypeId>) -> Result<()> {
    MESSAGE_BUS.register_handler(handler, msg_type)
}

/// Process existing messages on the bus, up to an optional limit
///
/// * limit: Option<usize> - optional limit
///
/// -> Result<()> - Ok if messages where able to be processed successfully
pub fn process_messages(limit: Option<usize>) -> Result<()> {
    MESSAGE_BUS.process_messages(limit)
}

struct Envelope {
    pub message: Box<dyn Message>,
    pub timestamp: Instant,
    pub id: u64,
}

enum HandlerType {
    Generic,
    Typed(TypeId),
}

struct Handler {
    inner: Box<dyn MessageHandler>,
    type_id: HandlerType,
}

pub struct MessageBus {
    sender: Sender<Envelope>,
    receiver: Receiver<Envelope>,
    handlers: RwLock<Vec<Handler>>,
    next_message_id: AtomicU64,
}

impl MessageBus {
    pub fn new(size: Option<usize>) -> Self {
        let (sender, receiver) = match size {
            Some(size) => bounded(size),
            None => unbounded(),
        };

        Self {
            sender,
            receiver,
            handlers: RwLock::new(Vec::new()),
            next_message_id: AtomicU64::new(0),
        }
    }

    pub fn register_handler(&self, handler: Box<dyn MessageHandler>, msg_type: Option<TypeId>) -> Result<()> {
        let type_id = match msg_type {
            Some(msg_type) => HandlerType::Typed(msg_type),
            None => HandlerType::Generic,
        };

        match self.handlers.write() {
            Ok(mut handlers) => {
                handlers.push(Handler {
                    inner: handler,
                    type_id,
                });
                Ok(())
            },
            Err(_) => Err(VtrlError::MessageBus("Unable to obtain lock on handler!".to_string())),
        }
    }

    pub fn send<M: Message + 'static>(&self, message: M) -> Result<u64> {
        let id = self.next_message_id.fetch_add(1, Ordering::SeqCst);

        let envelope = Envelope {
            message: Box::new(message),
            timestamp: Instant::now(),
            id,
        };

        self.sender.send(envelope).map_err(|_| VtrlError::MessageBus("Error sending message to bus!".to_string()))?;
        
        Ok(id)
    }

    pub fn process_messages(&self, limit: Option<usize>) -> Result<()> {
        let limit = limit.unwrap_or(usize::MAX);
        let mail: Vec<_> = self.receiver.try_iter().take(limit).collect();

        match self.handlers.write() {
            Ok(mut handlers) => {
                for envelope in mail {
                    log::trace!("Processing message {}", envelope.id);

                    if let Some(ttl) = envelope.message.ttl()
                        && Instant::now() - envelope.timestamp > ttl {
                            continue;
                    }

                    let type_id = envelope.message.message_type_id();

                    for handler in &mut *handlers {
                        match handler.type_id {
                            HandlerType::Generic => handler.inner.call(&*envelope.message),
                            HandlerType::Typed(tid) if tid == type_id => {
                                handler.inner.call(&*envelope.message)
                            },
                            _ => continue,
                        }
                    }
                }

                Ok(())
            },
            Err(_) => Err(VtrlError::MessageBus("Unable to obtain lock on handler!".to_string())),
        }
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        MessageBus::new(None)
    }
}
