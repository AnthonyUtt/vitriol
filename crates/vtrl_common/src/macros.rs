/// Appends a null character ('\0') to the end of a &str
/// to create a C-style string
#[macro_export]
macro_rules! c_str {
    ($str:ident) => {
        ([$str, "\0"].concat())
    };
}
