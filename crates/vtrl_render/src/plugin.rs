use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

use crate::font_atlas::*;
use crate::texture_atlas::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, world: &mut World, _: &mut AssetManager) {
        // world.add_resource(AnimationStore::default());
        //
        // world.add_system(ScheduleSlot::PreRender, |w, mgr| {
        //     // Drain newly-loaded AnimationSet assets into AnimationStore.
        //     // Runs in PreRender so the SceneManager system (in First) has
        //     // populated `just_loaded` by the time we read it. We take the
        //     // list out of SceneManager so its RefMut releases before we
        //     // re-borrow the world for AnimationStore.
        //     let mut loaded = match w.get_resource_mut::<SceneManager>() {
        //         Some(mut s) if !s.just_loaded.is_empty() => std::mem::take(&mut s.just_loaded),
        //         _ => return,
        //     };
        //
        //     {
        //         let mut store = w.get_resource_mut::<AnimationStore>().unwrap();
        //         loaded.retain(|(ty, sym)| {
        //             if ty == "AnimationSet" {
        //                 if let Some(set) = mgr.get::<AnimationSet>(*sym) {
        //                     for (name, frames) in &set.0 {
        //                         store.insert(name.clone(), frames.clone());
        //                     }
        //                 }
        //                 false
        //             } else {
        //                 true
        //             }
        //         });
        //     }
        //
        //     // Put back anything we didn't drain so other consumers can.
        //     if !loaded.is_empty() {
        //         w.get_resource_mut::<SceneManager>()
        //             .unwrap()
        //             .just_loaded
        //             .append(&mut loaded);
        //     }
        // });

        // Supplying the command buffer as a resource on the world
        // simplifies how different systems across the engine interact
        // with the renderer. While most systems should just update
        // entities to include components that will get picked up by
        // the primary render system (quads, sprites, etc), some systems
        // have specific things to render that would be better off remaining
        // uncoupled from the renderer plugin (i.e. debug rendering for colliders,
        // the main debug overlay, etc).
        world.add_resource(CommandBuffer::default);

        world.add_system(ScheduleSlot::Init, |w, _mgr| {
            // Global font + texture atlases for loading font data,
            // added on init so we can be sure that GLFW / GL have
            // both been initialized and loaded (since this runs
            // after the WindowPlugin).
            let font_atlas = FontAtlas::default();
            // TODO: load default font - need to add new load_from_bytes
            // method to asset manager
            w.add_resource(font_atlas);
            w.add_resource(TextureAtlas::new(1024, 1024))
        });

        // Primary render system
        // - Query all entities to be rendered
        // - build individual render passes
        // - submit to render queue
        world.add_system(ScheduleSlot::Render, |w, _| {
            let mut cb = w.get_resource_mut::<CommandBuffer>()
                .expect("Unable to find command buffer!");
            let viewport = w.get_resource::<Viewport>()
                .expect("Unable to find viewport!");
            
            let view_projection = w.view::<(Camera, Transform), ()>()
                .iter()
                .find(|(_, (cam, _))| cam.primary)
                .map(|(_, (cam, xform))| {
                    cam.view_projection(xform.position, xform.rotation, Vec2::new(viewport.width as f32, viewport.height as f32))
                });

            cb.push(RenderCommand::BeginPass {
                name: "world",
                target: RenderTarget::Screen,
                clear: Some(Vec4::new(0.3, 0.5, 0.6, 1.)),
                blend_mode: Some(BlendMode::Alpha),
                view_projection,
            });

            let view = w.view::<(Quad, Transform), ()>();
            let mut instances: Vec<QuadInstance> = Vec::new();
            for (_, (quad, xform)) in view.iter() {
                instances.push(QuadInstance {
                    pos: xform.position,
                    size: quad.size * xform.scale,
                    rot: xform.rotation,
                    z: xform.z_index,
                    color: quad.color,
                    uv: Vec4::zero(),
                    tex: 0.,
                });
            }

            cb.push(RenderCommand::DrawQuads { instances: instances.into() });

            // let view = w.view::<(Sprite, Transform), ()>();
            // let mut instances: Vec<QuadInstance> = Vec::new();
            // for (entity, (sprite, xform)) in view.iter() {
            //     let (tex_id, uv) = if w.has_component::<Animation>(entity) {
            //         let mut anim = w.get_component_mut::<Animation>(entity).unwrap();
            //         let tex_id: u32 = mgr
            //             .get::<Texture>(anim.texture_handle)
            //             .map(|t| t.id)
            //             .unwrap_or(0);
            //         let frames = animations
            //             .get(anim.active_animation.to_string())
            //             .unwrap_or_else(|| {
            //                 panic!("Animation not found! {}", anim.active_animation)
            //             });
            //         let frame = frames[anim.current_frame];
            //
            //         // update animation timing & frame if necessary
            //         anim.elapsed += dt;
            //         if anim.elapsed >= frame.duration {
            //             anim.elapsed = 0.;
            //             anim.current_frame += 1;
            //             if anim.current_frame >= frames.len() {
            //                 anim.current_frame = 0;
            //             }
            //         }
            //
            //         (tex_id, frames[anim.current_frame].uv)
            //     } else {
            //         let tex_id: u32 = mgr
            //             .get::<Texture>(sprite.texture_handle)
            //             .map(|t| t.id)
            //             .unwrap_or(0);
            //
            //         (tex_id, sprite.uv)
            //     };
            //     let uv = context::compute_uv(tex_id as usize, uv);
            //
            //     instances.push(QuadInstance {
            //         pos: xform.position,
            //         size: sprite.size * xform.scale,
            //         rot: xform.rotation,
            //         z: xform.z_index,
            //         color: sprite.color,
            //         uv,
            //         tex: tex_id as f32,
            //     });
            // }
            //
            // cb.push(RenderCommand::DrawQuads { instances: instances.into() });
        });

        // Post-render
        // - Re-organize draw calls based on passes/blending/etc
        // - Submit ordered draw calls to GPU in single batch
        world.add_system(ScheduleSlot::PostRender, |_, _| {

        });

        // Window plugin already handles swapping buffers in ScheduleSlot::Last
    }
}
