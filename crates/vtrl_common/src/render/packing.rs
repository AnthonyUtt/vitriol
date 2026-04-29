use ultraviolet::UVec2;

pub struct ShelfPacker {
    pub width: u32,
    pub height: u32,
    pub shelves: Vec<Shelf>,
}

pub struct Shelf {
    y: u32,
    height: u32,
    cursor_x: u32,
}

impl ShelfPacker {
    pub fn pack(&mut self, glyph_w: u32, glyph_h: u32) -> Option<UVec2> {
        for shelf in &mut self.shelves {
            if glyph_h <= shelf.height && shelf.cursor_x + glyph_w <= self.width {
                let pos = UVec2::new(shelf.cursor_x, shelf.y);
                shelf.cursor_x += glyph_w;
                return Some(pos);
            }
        }

        let y = self.shelves.last().map(|s| s.y + s.height).unwrap_or(0);

        if y + glyph_h > self.height {
            return None;
        }

        self.shelves.push(Shelf {
            y,
            height: glyph_h,
            cursor_x: glyph_w,
        });

        Some(UVec2::new(0, y))
    }
}
