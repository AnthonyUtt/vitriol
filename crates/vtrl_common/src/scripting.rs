pub trait Scriptable {
    fn register_script_api(engine: &mut rhai::Engine);
}

pub struct ScriptableRegistration {
    pub register_fn: fn(&mut rhai::Engine),
}

impl ScriptableRegistration {
    pub const fn new(register_fn: fn(&mut rhai::Engine)) -> Self {
        Self { register_fn }
    }
}

inventory::collect!(ScriptableRegistration);
