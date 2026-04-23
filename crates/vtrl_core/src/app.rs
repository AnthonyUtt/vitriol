use std::{
    any::TypeId,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use vtrl_common::prelude::*;

pub struct App {
    should_close: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> App {
        env_logger::init();
        log::info!("Initializing VITRIOL Engine...");

        let should_close = Arc::new(AtomicBool::new(false));

        let sc = should_close.clone();
        ctrlc::set_handler(move || {
            log::trace!("Ctrl-C detected, shutting down!");
            sc.store(true, Ordering::SeqCst);
        })
        .expect("Unable to set Ctrl-C handler!");

        let sc = should_close.clone();
        let handler = SystemMessageHandler { should_close: sc };
        message_bus::register_handler(Box::new(handler), Some(TypeId::of::<SystemMessage>()))
            .expect("Unable to register system message handler!");

        #[cfg(debug_assertions)]
        message_bus::register_handler(Box::new(MessageSink), None)
            .expect("Unable to register message sink!");

        App { should_close }
    }

    pub fn run(&self) -> Result<()> {
        loop {
            if self.should_close.load(Ordering::SeqCst) {
                break;
            }

            message_bus::process_messages(None)?;
        }

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

struct SystemMessageHandler {
    should_close: Arc<AtomicBool>,
}
impl MessageHandler for SystemMessageHandler {
    fn call(&self, msg: &dyn Message) {
        if let Some(msg) = msg.as_any().downcast_ref::<SystemMessage>() {
            match msg {
                SystemMessage::Shutdown => self.should_close.store(true, Ordering::SeqCst),
                SystemMessage::Ping => {}
            }
        }
    }
}

struct MessageSink;
impl MessageHandler for MessageSink {
    fn call(&self, msg: &dyn Message) {
        log::trace!("{msg:?}");
    }
}
