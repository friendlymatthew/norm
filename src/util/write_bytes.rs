#[macro_export]
macro_rules! write {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, field: $type) -> Result<()> {
            self.data.extend_from_slice(&field.to_be_bytes());

            Ok(())
        }
    };
}
