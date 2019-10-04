pub mod spanned_value;
mod spanned;
mod map;

pub use spanned_value::SpannedValue;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
