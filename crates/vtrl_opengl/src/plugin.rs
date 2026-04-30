use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;
use vtrl_plugins::prelude::*;

use crate::animations::*;
use crate::context;
use crate::primitives::*;

pub struct Renderer2DPlugin;

impl Plugin for Renderer2DPlugin {
    fn build(&self, world: &mut World, _mgr: &mut AssetManager) {
        world.add_resource(AnimationStore::default());

        world.add_system(ScheduleSlot::PreRender, |w, mgr| {
            // Drain newly-loaded AnimationSet assets into AnimationStore.
            // Runs in PreRender so the SceneManager system (in First) has
            // populated `just_loaded` by the time we read it. We take the
            // list out of SceneManager so its RefMut releases before we
            // re-borrow the world for AnimationStore.
            let mut loaded = match w.get_resource_mut::<SceneManager>() {
                Some(mut s) if !s.just_loaded.is_empty() => std::mem::take(&mut s.just_loaded),
                _ => return,
            };

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

            // Put back anything we didn't drain so other consumers can.
            if !loaded.is_empty() {
                w.get_resource_mut::<SceneManager>()
                    .unwrap()
                    .just_loaded
                    .append(&mut loaded);
            }
        });

        world.add_system(ScheduleSlot::Render, |w, mgr| {
            let animations = w.get_resource::<AnimationStore>().unwrap();
            let dt = w.get_resource::<DeltaTime>().map(|t| t.0).unwrap_or(0.);

            context::push_command(RenderCommand::BeginPass {
                name: "world",
                target: RenderTarget::Screen,
                clear: Some(Vec4::new(0.3, 0.5, 0.6, 1.)),
                blend_mode: Some(BlendMode::Alpha),
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

            context::push_command(RenderCommand::DrawQuads {
                instances: instances.into(),
            });

            let view = w.view::<(Sprite, Transform), ()>();
            let mut instances: Vec<QuadInstance> = Vec::new();
            for (entity, (sprite, xform)) in view.iter() {
                let (tex_id, uv) = if w.has_component::<Animation>(entity) {
                    let mut anim = w.get_component_mut::<Animation>(entity).unwrap();
                    let tex_id: u32 = mgr
                        .get::<Texture>(anim.texture_handle)
                        .map(|t| t.id)
                        .unwrap_or(0);
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

                    (tex_id, frames[anim.current_frame].uv)
                } else {
                    let tex_id: u32 = mgr
                        .get::<Texture>(sprite.texture_handle)
                        .map(|t| t.id)
                        .unwrap_or(0);

                    (tex_id, sprite.uv)
                };
                let uv = context::compute_uv(tex_id as usize, uv);

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
            context::push_command(RenderCommand::DrawQuads {
                instances: instances.into(),
            });

            context::push_command(RenderCommand::BeginPass {
                name: "text",
                target: RenderTarget::Screen,
                clear: None,
                blend_mode: Some(BlendMode::PremultipliedAlpha),
            });

            let view = w.view::<(Text, Transform), ()>();
            let mut instances: Vec<GlyphInstance> = Vec::new();
            for (_, (text, xform)) in view.iter() {
                instances.extend(layout_text(&text, &xform));
            }

            context::push_command(RenderCommand::DrawText {
                instances: instances.into(),
            });
        });

        world.add_system(ScheduleSlot::Last, |_, _| {
            context::process_queue();
        });
    }
}

pub struct DebugOverlayPlugin {
    pub anchor: Anchor,
    pub style: TextStyle,
    pub padding: Vec2,
    pub color: Vec4,
}

impl Default for DebugOverlayPlugin {
    fn default() -> Self {
        Self {
            anchor: Anchor::TopLeft,
            style: TextStyle::default(),
            padding: Vec2::new(8.0, 8.0),
            color: Vec4::one(),
        }
    }
}

impl Plugin for DebugOverlayPlugin {
    fn build(&self, world: &mut World, _mgr: &mut AssetManager) {
        let anchor = self.anchor;
        let style = self.style;
        let padding = self.padding;
        let color = self.color;

        world.add_system(ScheduleSlot::PostRender, move |_, _| {
            let lines = vtrl_common::debug::drain_lines();
            let instances = layout_overlay(&lines, anchor, style, padding, color);

            context::push_command(RenderCommand::BeginPass {
                name: "debug",
                target: RenderTarget::Screen,
                clear: None,
                blend_mode: Some(BlendMode::PremultipliedAlpha),
            });
            context::push_command(RenderCommand::DrawText {
                instances: instances.into(),
            });
        });
    }
}

fn layout_text(_text: &Text, _xform: &Transform) -> Vec<GlyphInstance> {
    // TODO: walk glyphs in `text.text`, splitting on '\n', advancing by
    // glyph metrics horizontally and `style.line_height * style.size`
    // vertically. Use the FreeType-backed font atlas to resolve UVs and
    // sizes per glyph. Apply `xform.position`, `xform.scale`, `xform.z_index`,
    // and `text.color`.
    todo!()
}

fn layout_overlay(
    lines: &[String],
    anchor: Anchor,
    style: TextStyle,
    padding: Vec2,
    color: Vec4,
) -> Vec<GlyphInstance> {
    if lines.is_empty() {
        return Vec::new();
    }

    let logical_lines: Vec<&str> = lines.iter().flat_map(|s| s.split('\n')).collect();
    let line_advance = style.size * style.line_height;
    let total_height = line_advance * logical_lines.len() as f32;
    let window = context::window_size();

    // Right-anchored variants need each line's advance-sum to push the pen
    // leftward off the right edge before drawing.
    let line_widths: Vec<f32> = match anchor {
        Anchor::TopRight | Anchor::BottomRight => logical_lines
            .iter()
            .map(|l| measure_line_width(l, style.font_id))
            .collect(),
        _ => Vec::new(),
    };

    // Baselines descend from this y; style.size approximates the ascent and
    // assumes the font was rasterized at roughly that pixel size.
    let first_baseline = match anchor {
        Anchor::TopLeft | Anchor::TopRight => padding.y + style.size,
        Anchor::BottomLeft | Anchor::BottomRight => {
            window.y - padding.y - total_height + style.size
        }
    };

    let mut instances = Vec::with_capacity(logical_lines.iter().map(|l| l.len()).sum());
    for (i, line) in logical_lines.iter().enumerate() {
        let pen_start = match anchor {
            Anchor::TopLeft | Anchor::BottomLeft => padding.x,
            Anchor::TopRight | Anchor::BottomRight => window.x - padding.x - line_widths[i],
        };
        let baseline = first_baseline + i as f32 * line_advance;
        let mut pen_x = pen_start;

        for c in line.chars() {
            let Some(glyph) = context::get_glyph(style.font_id, c) else {
                continue;
            };

            // Whitespace glyphs (space, tab) carry no bitmap but still advance.
            if glyph.width > 0 && glyph.height > 0 {
                instances.push(GlyphInstance {
                    pos: Vec2::new(pen_x + glyph.left as f32, baseline - glyph.top as f32),
                    size: Vec2::new(glyph.width as f32, glyph.height as f32),
                    rot: 0.0,
                    z: 1.0,
                    color,
                    uv: glyph.uv,
                    tex: style.font_id as f32,
                });
            }

            pen_x += glyph.advance_x as f32;
        }
    }

    instances
}

fn measure_line_width(line: &str, font_id: u32) -> f32 {
    line.chars()
        .filter_map(|c| context::get_glyph(font_id, c))
        .map(|g| g.advance_x as f32)
        .sum()
}
