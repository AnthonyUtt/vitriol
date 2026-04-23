use std::any::{Any, TypeId, type_name};
use std::fmt::Debug;
use std::time::Duration;

pub trait MessageHandler: Send + Sync {
    fn call(&self, evt: &dyn Message);
}

/// A message that can be sent on the message bus
pub trait Message
where
    Self: 'static + Send + Sync + Debug,
{
    /// Get the type ID of the message
    fn message_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Get the type name of the message
    fn message_type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    /// Get the priority of the message
    fn priority(&self) -> u32 {
        0 // default priority
    }

    /// Get whether the message requires an acknowledgement
    fn requires_ack(&self) -> bool {
        false
    }

    /// Get the time-to-live of the message
    fn ttl(&self) -> Option<Duration> {
        None
    }

    /// Get the category of the message
    fn category(&self) -> Option<&str> {
        None
    }

    /// Downcast the message to `Any`
    fn as_any(&self) -> &dyn Any;
    /// Downcast the message to `Any` mutably
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub enum SystemMessage {
    Ping,
    Shutdown,
}

impl Message for SystemMessage {
    fn category(&self) -> Option<&str> {
        Some("system")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
