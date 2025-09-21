#[macro_export]
macro_rules! impl_read_for_datatype {
    ($name:ident, $type:ty) => {
        fn $name(&mut self) -> Result<$type> {
            let width = std::mem::size_of::<$type>();
            let slice = self.read_slice(width)?;

            Ok(<$type>::from_be_bytes(slice.try_into()?))
        }
    };
}

#[macro_export]
macro_rules! impl_read_slice {
    () => {
        #[allow(dead_code)]
        fn read_vec<T>(
            &mut self,
            capacity: usize,
            read_fn: impl Fn(&mut Self) -> Result<T>,
        ) -> Result<Vec<T>> {
            (0..capacity)
                .map(|_| read_fn(self))
                .collect::<Result<Vec<_>>>()
        }

        fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
            let slice = self
                .data
                .get(self.cursor..self.cursor + len)
                .ok_or_else(|| anyhow::anyhow!("oob"))?;

            self.cursor += len;

            Ok(slice)
        }

        #[allow(dead_code)]
        fn peek_slice(&mut self, len: usize) -> Result<&'a [u8]> {
            self.data
                .get(self.cursor..self.cursor + len)
                .ok_or_else(|| anyhow::anyhow!("oob"))
        }
    };
}
