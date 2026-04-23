use std::{
    any::TypeId,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

use crate::schedule::*;

pub struct App {
    should_close: Arc<AtomicBool>,
    schedule: Schedule,
    world: World,
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

        let mut app = App {
            should_close,
            schedule: Schedule::default(),
            world: World::new()
        };

        app.bootstrap();

        app
    }

    pub fn run(&mut self) -> Result<()> {
        // init
        self.run_stage(ScheduleSlot::PreInit);
        self.run_stage(ScheduleSlot::Init);
        self.run_stage(ScheduleSlot::PostInit);

        // main loop
        loop {
            if self.should_close.load(Ordering::SeqCst) {
                break;
            }

            self.run_stage(ScheduleSlot::First);

            self.run_stage(ScheduleSlot::PreUpdate);
            self.run_stage(ScheduleSlot::Update);
            self.run_stage(ScheduleSlot::PostUpdate);

            self.run_stage(ScheduleSlot::PreFixedUpdate);
            self.run_stage(ScheduleSlot::FixedUpdate);
            self.run_stage(ScheduleSlot::PostFixedUpdate);

            self.run_stage(ScheduleSlot::Last);
        }

        // cleanup
        self.run_stage(ScheduleSlot::PreShutdown);
        self.run_stage(ScheduleSlot::Shutdown);
        self.run_stage(ScheduleSlot::PostShutdown);
        
        Ok(())
    }

    pub fn with_service(mut self, slot: ScheduleSlot, service: impl Service) -> Self {
        self.schedule.add_service(slot, service);
        self
    }

    fn run_stage(&mut self, slot: ScheduleSlot) {
        let services = self.schedule.services_for_slot(slot);
        for service in services.iter() {
            service(&mut self.world);
        }
    }

    fn bootstrap(&mut self) {
        self.world.add_resource(LastFrameTime(Instant::now()));
        self.world.add_resource(DeltaTime(0.0));
        self.world.add_resource(FrameRate(0.0));

        self.schedule.add_service(ScheduleSlot::First, |w| {
            let mut lft = w.get_resource_mut::<LastFrameTime>().unwrap();
            let mut dt = w.get_resource_mut::<DeltaTime>().unwrap();
            let mut fps = w.get_resource_mut::<FrameRate>().unwrap();

            let current_frame_time = Instant::now();
            let new_dt = current_frame_time - lft.0;
            dt.0 = new_dt.as_secs_f32();
            lft.0 = current_frame_time;
            fps.0 = 1.0 / dt.0;

            log::info!("FPS: {}", fps.0);
        });

        self.schedule.add_service(ScheduleSlot::Last, |_| {
            let _ = message_bus::process_messages(None);
        });
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

struct LastFrameTime(pub Instant);
struct DeltaTime(pub f32);
struct FrameRate(pub f32);
