use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;
use vtrl_render::prelude::*;

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

        world.add_system(ScheduleSlot::Render, move |w, _| {
            let mut cb = w.get_resource_mut::<CommandBuffer>()
                .expect("Unable to find render command buffer!");
            let viewport = w.get_resource::<Viewport>()
                .expect("Unable to find viewport info!");
            let fonts = w.get_resource::<FontAtlas>()
                .expect("Unable to find font store!");

            let matrix = ortho_top_left_matrix(viewport.width as f32, viewport.height as f32);

            let lines = vtrl_common::debug::drain_lines();
            let instances = layout_overlay(
                &lines,
                anchor,
                style,
                padding,
                color,
                Vec2::new(viewport.width as f32, viewport.height as f32),
                &fonts,
            );

            cb.push(RenderCommand::BeginPass {
                name: "debug",
                target: RenderTarget::Screen,
                clear: None,
                blend_mode: Some(BlendMode::PremultipliedAlpha),
                view_projection: Some(matrix),
            });

            cb.push(RenderCommand::DrawText {
                instances: instances.into(),
            });
        });
    }
}

fn layout_overlay(
    lines: &[String],
    anchor: Anchor,
    style: TextStyle,
    padding: Vec2,
    color: Vec4,
    window: Vec2,
    fonts: &FontAtlas,
) -> Vec<GlyphInstance> {
    if lines.is_empty() {
        return Vec::new();
    }

    let debug_font = fonts.get_debug_font();
    if debug_font.is_none() {
        return Vec::new();
    }
    let debug_font = debug_font.unwrap();

    let logical_lines: Vec<&str> = lines.iter().flat_map(|s| s.split('\n')).collect();
    let line_advance = style.size * style.line_height;
    let total_height = line_advance * logical_lines.len() as f32;

    // Right-anchored variants need each line's advance-sum to push the pen
    // leftward off the right edge before drawing.
    let line_widths: Vec<f32> = match anchor {
        Anchor::TopRight | Anchor::BottomRight => logical_lines
            .iter()
            .map(|l| measure_line_width(l, debug_font, fonts))
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
            let Some(glyph) = fonts.get_glyph(debug_font, c) else {
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
                    tex: *fonts.get_font_tex_id(debug_font).unwrap() as f32,
                });
            }

            pen_x += glyph.advance_x as f32;
        }
    }

    instances
}

fn measure_line_width(line: &str, font: AssetHandle, fonts: &FontAtlas) -> f32 {
    line.chars()
        .filter_map(|c| fonts.get_glyph(font, c))
        .map(|g| g.advance_x as f32)
        .sum()
}

// Matrix to be submitted to shader for converting pixels to NDC
fn ortho_top_left_matrix(width: f32, height: f32) -> Mat4 {
    let near = -1.;
    let far = 1.;
    let sx = 2. / width;
    let sy = -2. / height; // negative to flip Y axis
    let sz = 2. / (far - near);
    let tx = -1.;
    let ty = 1.;
    let tz = -(far + near) / (far - near);

    Mat4 {
        cols: [
            Vec4::from([sx, 0., 0., 0.]),
            Vec4::from([0., sy, 0., 0.]),
            Vec4::from([0., 0., sz, 0.]),
            Vec4::from([tx, ty, tz, 1.]),
        ],
    }
}
