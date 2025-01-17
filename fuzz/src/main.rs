use afl::fuzz;
use png::Decoder;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut decoder = Decoder::new(data);
        let _ = decoder.decode();
    });
}
