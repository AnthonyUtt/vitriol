use std::{
    any::TypeId,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;
#[cfg(debug_assertions)]
use vtrl_opengl::plugin::DebugOverlayPlugin;
use vtrl_opengl::{plugin::Renderer2DPlugin, prelude::*};
use vtrl_plugins::prelude::*;

use crate::plugin::*;

pub struct App {
    should_close: Arc<AtomicBool>,
    plugins: PluginStorage,
    world: World,
    assets: AssetManager,
}

impl App {
    pub fn new() -> App {
        {
            dotenvy::dotenv().ok();
            use env_logger::Env;

            #[cfg(debug_assertions)]
            let filter = "warn,vitriol=debug,vtrl=debug";

            #[cfg(not(debug_assertions))]
            let filter = "warn";

            let env = Env::default().default_filter_or(filter);
            let mut logger = env_logger::Builder::from_env(env);
            logger.init();
        }

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

        App {
            should_close,
            plugins: PluginStorage::default(),
            world: World::new(),
            assets: AssetManager::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Bootstrap must be run just before the schedule starts
        self.bootstrap();

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

            self.run_stage(ScheduleSlot::PreRender);
            self.run_stage(ScheduleSlot::Render);
            self.run_stage(ScheduleSlot::PostRender);

            self.run_stage(ScheduleSlot::Last);
        }

        // cleanup
        self.run_stage(ScheduleSlot::PreShutdown);
        self.run_stage(ScheduleSlot::Shutdown);
        self.run_stage(ScheduleSlot::PostShutdown);

        Ok(())
    }

    pub fn with_plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins.insert(plugin);
        self
    }

    pub fn with_default_plugins(mut self) -> Self {
        self.plugins.insert(SceneManagerPlugin);
        self.plugins.insert(Renderer2DPlugin);
        self.plugins.insert(TimePlugin);
        self.plugins.insert(InputPlugin);
        #[cfg(debug_assertions)]
        self.plugins.insert(DebugOverlayPlugin::default());
        self
    }

    pub fn with_system(mut self, slot: ScheduleSlot, system: impl System) -> Self {
        self.world.add_system(slot, system);
        self
    }

    fn run_stage(&mut self, slot: ScheduleSlot) {
        let systems = self.world.systems_for_slot(slot);
        for system in systems.iter() {
            system(&mut self.world, &mut self.assets);
        }
    }

    fn bootstrap(&mut self) {
        render_context::init(WindowSettings::default())
            .expect("Unable to initialize render context!");
        self.plugins.bootstrap(&mut self.world, &mut self.assets);

        self.world.add_system(ScheduleSlot::Last, |_, _| {
            render_context::process_events();
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

#[allow(dead_code)]
struct MessageSink;
impl MessageHandler for MessageSink {
    fn call(&self, msg: &dyn Message) {
        log::trace!("{msg:?}");
    }
}
