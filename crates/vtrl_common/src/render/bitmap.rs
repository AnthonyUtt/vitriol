use serde_derive::*;

/// Naive bitmap implementation
///
/// A Bitmap struct that takes a type parameter `T` and
/// a color channel count `N`. For example, an RGBA bitmap
/// with an underlying u8 type would be represented as
/// Bitmap<u8, 4>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bitmap<T: Default + Copy, const N: usize> {
    /// The buffer containing the values of the bitmap
    pub buffer: Vec<T>,
    /// The width of the bitmap
    pub width: u32,
    /// The height of the bitmap
    pub height: u32,
}
impl<T: Default + Copy, const N: usize> Bitmap<T, N> {
    /// Creates a new Bitmap<T, N> of the given width and height.
    pub fn new(width: u32, height: u32) -> Bitmap<T, N> {
        let buffer_size = N * (width * height) as usize;
        let buffer = vec![T::default(); buffer_size];
        Bitmap {
            buffer,
            width,
            height,
        }
    }

    /// Creates a new Bitmap<T, N> from the underlying raw buffer
    pub fn new_from_data(buffer: Vec<T>, width: u32, height: u32) -> Bitmap<T, N> {
        debug_assert_eq!(buffer.len(), width as usize * height as usize * N);

        Bitmap {
            buffer,
            width,
            height,
        }
    }

    /// Inserts values from a given buffer into the bitmap
    ///
    /// Inserts values from the buffer into a rect inside the
    /// bitmap. The bounds of the rect are defined by the x, y,
    /// width, and height parameters.
    pub fn put(&mut self, buffer: &[T], x: u32, y: u32, width: u32, height: u32) {
        debug_assert!(x + width <= self.width);
        debug_assert!(y + height <= self.height);
        debug_assert_eq!(buffer.len(), N * width as usize * height as usize);

        let start_index: usize = x as usize + y as usize * self.width as usize * N;

        for row in 0..(N * height as usize) {
            for pixel in 0..(N * width as usize) {
                let dst_index = start_index + pixel + row * self.width as usize * N;
                let src_index = pixel + row * width as usize * N;
                self.buffer[dst_index] = buffer[src_index];
            }
        }
    }

    /// Retrieves a given rect from the bitmap
    pub fn get(&self, x: u32, y: u32, width: u32, height: u32) -> Vec<T> {
        debug_assert!(x + width <= self.width);
        debug_assert!(y + height <= self.height);

        let mut buffer = vec![T::default(); N * (width * height) as usize];
        let start_index: usize = x as usize + y as usize * self.width as usize * N;

        for row in 0..(N * height as usize) {
            for pixel in 0..(N * width as usize) {
                let dst_index = pixel + row * width as usize * N;
                let src_index = start_index + pixel + row * self.width as usize * N;
                buffer[dst_index] = self.buffer[src_index];
            }
        }

        buffer
    }
}
