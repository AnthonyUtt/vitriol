use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

use crate::context;

pub struct Renderer2DPlugin;

impl Plugin for Renderer2DPlugin {
    fn build(&self, world: &mut World) {
        world.add_system(ScheduleSlot::PostUpdate, |w, _| {
            let view = w.view::<QuadComponent, ()>();
            let mut instances: Vec<QuadInstance> = Vec::new();
            for (_, quad) in view.iter() {
                let uv = context::compute_uv(quad.texture_id as usize, quad.uv);
                instances.push(QuadInstance {
                    pos: quad.position,
                    size: quad.size,
                    rot: quad.rotation,
                    z: quad.z_index,
                    color: quad.color,
                    uv,
                    tex: quad.texture_id as f32,
                });
            }

            context::draw_quad_instances(instances.as_slice());
        });

        world.add_system(ScheduleSlot::PostUpdate, |w, _| {
            let view = w.view::<(TextComponent, Transform), ()>();
            let mut instances: Vec<GlyphInstance> = Vec::new();
            for (_, (text, xform)) in view.iter() {
                instances.extend(layout_text(&text, &xform));
            }

            context::draw_text_instances(instances.as_slice());
        });

        world.add_system(ScheduleSlot::PreUpdate, |_, _| {
            context::clear(0.5, 0.3, 0.7, 1.);
        });
    }
}

#[derive(Component)]
pub struct QuadComponent {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub z_index: f32,
    pub color: Vec4,
    pub uv: Vec4,
    pub texture_id: u32,
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
    fn build(&self, world: &mut World) {
        let anchor = self.anchor;
        let style = self.style;
        let padding = self.padding;
        let color = self.color;

        world.add_system(ScheduleSlot::Last, move |_, _| {
            let lines = vtrl_common::debug::drain_lines();
            let instances = layout_overlay(&lines, anchor, style, padding, color);
            context::draw_text_instances(instances.as_slice());
        });
    }
}

fn layout_text(_text: &TextComponent, _xform: &Transform) -> Vec<GlyphInstance> {
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
            Anchor::TopRight | Anchor::BottomRight => {
                window.x - padding.x - line_widths[i]
            }
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
