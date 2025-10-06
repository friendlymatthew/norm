use afl::fuzz;
use norm::png::PngDecoder;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut decoder = PngDecoder::new(data);
        let _ = decoder.decode();
    });
}
