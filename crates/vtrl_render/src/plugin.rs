use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;
use vtrl_plugins::prelude::*;
use vtrl_opengl::prelude::*;

use crate::animations::*;
use crate::font_atlas::*;
use crate::texture_atlas::*;
use crate::renderer::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, world: &mut World, _: &mut AssetManager) {
        // Supplying the command buffer as a resource on the world
        // simplifies how different systems across the engine interact
        // with the renderer. While most systems should just update
        // entities to include components that will get picked up by
        // the primary render system (quads, sprites, etc), some systems
        // have specific things to render that would be better off remaining
        // uncoupled from the renderer plugin (i.e. debug rendering for colliders,
        // the main debug overlay, etc).
        world.add_resource(CommandBuffer::default());
        world.add_resource(AnimationStore::default());

        world.add_system(ScheduleSlot::Init, |w, _mgr| {
            let mut font_atlas = FontAtlas::default();
            let debug_font_bytes = include_bytes!("./assets/monogram-extended.ttf");
            let debug_font = Font::load(debug_font_bytes.to_vec())
                .expect("Unable to parse debug font!");
            let debug_font_handle = AssetHandle::from(interned("debug-font.ttf"));
            font_atlas.set_debug_font(debug_font_handle, debug_font.glyphs)
                .expect("Unable to register debug font!");

            // Global font + texture atlases for loading font data,
            // added on init so we can be sure that GLFW / GL have
            // both been initialized and loaded (since this runs
            // after the WindowPlugin).
            w.add_resource(font_atlas);
            w.add_resource(TextureAtlas::new(1024, 1024));

            w.add_resource(Renderer::new());
        });

        world.add_system(ScheduleSlot::PreRender, |w, mgr| {
            // Drain newly-loaded assets into the necessary locations to
            // be used for rendering this frame. Runs in PreRender so the
            // SceneManager system (in First) should be have populated the
            // `just_loaded` vec by the time we get here.
            let mut loaded = match w.get_resource_mut::<SceneManager>() {
                Some(mut s) if !s.just_loaded.is_empty() => std::mem::take(&mut s.just_loaded),
                _ => return,
            };

            // Animations
            {
                let mut store = w.get_resource_mut::<AnimationStore>().unwrap();
                loaded.retain(|(ty, sym)| {
                    if ty == "AnimationSet" {
                        if let Some(set) = mgr.get::<AnimationSet>(*sym) {
                            for (name, frames) in &set.0 {
                                store.insert(name.clone(), frames.clone());
                            }
                        }
                        false
                    } else {
                        true
                    }
                });
            }

            {
                let mut atlas = w.get_resource_mut::<FontAtlas>().unwrap();
                loaded.retain(|(ty, sym)| {
                    if ty == "Font" {
                        if let Some(font) = mgr.get::<Font>(*sym) {
                            let _ = atlas.register_font(AssetHandle::from(*sym), font.glyphs.clone());
                        }
                        false
                    } else {
                        true
                    }
                })
            }

            {
                let mut atlas = w.get_resource_mut::<TextureAtlas>().unwrap();
                loaded.retain(|(ty, sym)| {
                    if ty == "TextureData" {
                        if let Some(tex) = mgr.get::<TextureData>(*sym) {
                            let _ = atlas.register_texture(AssetHandle::from(*sym), tex);
                        }
                        false
                    } else {
                        true
                    }
                })
            }

            // Put back anything we didn't drain so other consumers can.
            if !loaded.is_empty() {
                w.get_resource_mut::<SceneManager>()
                    .unwrap()
                    .just_loaded
                    .append(&mut loaded);
            }
        });

        // Primary render system
        // - Query all entities to be rendered
        // - build individual render passes
        // - submit to render queue
        world.add_system(ScheduleSlot::Render, |w, _| {
            let dt = w.get_resource::<DeltaTime>().unwrap().0;
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

            // let font_atlas = w.get_resource::<FontAtlas>().unwrap();
            let texture_atlas = w.get_resource::<TextureAtlas>().unwrap();
            let animations = w.get_resource::<AnimationStore>().unwrap();

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

            let view = w.view::<(Sprite, Transform), ()>();
            let mut instances: Vec<QuadInstance> = Vec::new();
            for (entity, (sprite, xform)) in view.iter() {
                let (tex_id, uv) = if w.has_component::<Animation>(entity) {
                    let mut anim = w.get_component_mut::<Animation>(entity).unwrap();
                    let tex_id = texture_atlas.get_texture_id(anim.texture_handle)
                        .cloned().unwrap_or(0);
                    let frames = animations
                        .get(anim.active_animation.to_string())
                        .unwrap_or_else(|| {
                            panic!("Animation not found! {}", anim.active_animation)
                        });
                    let frame = frames[anim.current_frame];

                    // update animation timing & frame if necessary
                    anim.elapsed += dt;
                    if anim.elapsed >= frame.duration {
                        anim.elapsed = 0.;
                        anim.current_frame += 1;
                        if anim.current_frame >= frames.len() {
                            anim.current_frame = 0;
                        }
                    }
                    
                    let scalar = texture_atlas.get_uv_scalar(anim.texture_handle);
                    let uv = frames[anim.current_frame].uv;
                    let uv = Vec4::new(
                        uv.x * scalar.x,
                        uv.y * scalar.y,
                        uv.z * scalar.x,
                        uv.w * scalar.y,
                    );

                    (tex_id, uv)
                } else {
                    let tex_id = texture_atlas.get_texture_id(sprite.texture_handle)
                        .cloned().unwrap_or(0);

                    let scalar = texture_atlas.get_uv_scalar(sprite.texture_handle);
                    let uv = sprite.uv;
                    let uv = Vec4::new(
                        uv.x * scalar.x,
                        uv.y * scalar.y,
                        uv.z * scalar.x,
                        uv.w * scalar.y,
                    );

                    (tex_id, uv)
                };

                instances.push(QuadInstance {
                    pos: xform.position,
                    size: sprite.size * xform.scale,
                    rot: xform.rotation,
                    z: xform.z_index,
                    color: sprite.color,
                    uv,
                    tex: tex_id as f32,
                });
            }

            cb.push(RenderCommand::DrawQuads { instances: instances.into() });
        });

        // Post-render
        // - Re-organize draw calls based on passes/blending/etc
        // - Submit ordered draw calls to GPU in single batch
        world.add_system(ScheduleSlot::PostRender, |w, _| {
            let commands = w.get_resource_mut::<CommandBuffer>()
                .unwrap().take();
            let r = w.get_resource_mut::<Renderer>().unwrap();

            let mut matrix = Mat4::identity();
            let tex_atlas = w.get_resource::<TextureAtlas>().unwrap();
            let font_atlas = w.get_resource::<FontAtlas>().unwrap();

            for cmd in commands.iter() {
                match cmd {
                    RenderCommand::BeginPass {
                        name: _,
                        target,
                        clear,
                        blend_mode,
                        view_projection,
                    } => {
                        match target {
                            RenderTarget::Screen => {
                                commands::bind_framebuffer(0);
                            },
                            RenderTarget::Framebuffer(id) => {
                                commands::bind_framebuffer(*id);
                            },
                        }

                        if let Some(color) = clear {
                            commands::clear(*color);
                        }

                        if let Some(mode) = blend_mode {
                            commands::set_blend_mode(*mode);
                        }

                        if let Some(mat) = view_projection {
                            matrix = *mat;
                        }
                    },
                    RenderCommand::DrawQuads { instances } => {
                        r.draw_quad_instances(
                            matrix,
                            &tex_atlas,
                            &font_atlas,
                            instances,
                        );
                    },
                    RenderCommand::DrawText { instances } => {
                        r.draw_text_instances(
                            matrix,
                            &tex_atlas,
                            &font_atlas,
                            instances,
                        );
                    },
                    RenderCommand::DrawLines { instances } => {
                        r.draw_line_instances(
                            matrix,
                            &tex_atlas,
                            &font_atlas,
                            instances,
                        );
                    },
                    RenderCommand::DrawCircles { instances } => {
                        r.draw_circle_instances(
                            matrix,
                            &tex_atlas,
                            &font_atlas,
                            instances,
                        );
                    },
                    _ => {},
                }
            }
        });

        // Window plugin already handles swapping buffers in ScheduleSlot::Last
    }
}
