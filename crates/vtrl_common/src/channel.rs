// Wrapper for channel crate

pub type Sender<T> = flume::Sender<T>;
pub type Receiver<T> = flume::Receiver<T>;

pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    flume::bounded::<T>(cap)
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    flume::unbounded::<T>()
}
