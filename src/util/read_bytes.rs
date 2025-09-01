pub const U8_BYTES: usize = size_of::<u8>();
pub const U16_BYTES: usize = size_of::<u16>();
pub const U32_BYTES: usize = size_of::<u32>();
pub const U64_BYTES: usize = size_of::<u64>();

#[macro_export]
macro_rules! read {
    ($name:ident, $type:ty, $size:expr) => {
        fn $name(&mut self) -> Result<$type> {
            let slice = self
                .data
                .get(self.cursor..self.cursor + $size)
                .ok_or_else(|| anyhow::anyhow!("oob"))?;

            let b = <$type>::from_be_bytes(slice.try_into()?);
            self.cursor += $size;

            Ok(b)
        }
    };
}

#[macro_export]
macro_rules! eof {
    () => {
        fn eof(&self, len: usize) -> Result<()> {
            let end = self.data.len();

            ensure!(
                self.cursor + len.saturating_sub(1) < self.data.len(),
                "Unexpected EOF. At {}, seek by {}, buffer size: {}.",
                self.cursor,
                len,
                end
            );

            Ok(())
        }
    };
}

#[macro_export]
macro_rules! read_slice {
    () => {
        fn read_fixed<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
            let bs = self
                .data
                .get(self.cursor..self.cursor + N)
                .ok_or_else(|| anyhow::anyhow!("oob"))?;

            self.cursor += N;

            Ok(bs.try_into()?)
        }

        fn read_vec<T>(
            &mut self,
            capacity: usize,
            read_fn: impl Fn(&mut Self) -> Result<T>,
        ) -> Result<Vec<T>> {
            let mut list = Vec::with_capacity(capacity);

            for _ in 0..capacity {
                list.push(read_fn(self)?)
            }

            Ok(list)
        }

        fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
            let slice = self
                .data
                .get(self.cursor..self.cursor + len)
                .ok_or_else(|| anyhow::anyhow!("oob"))?;
            self.cursor += len;

            Ok(slice)
        }
    };
}
