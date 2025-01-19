# png

This project aims to provide a PNG decoder backed by a feature-rich GPU-based renderer.

As a decoder, this project uses the [PNG test suite](http://www.schaik.com/pngsuite/) to validate its ability to handle
various PNG features and edge cases. Currently, `png` can decode and render images with an 8-bit color depth.

## Usage

To render an image, run `cargo run <image_path>`. For example:

```bash
# cd png
cargo run ./obama.png
```

### Additional Scripts

```bash
# Profile the decoder
./profile.sh ./reagan.png

# Run the PNG test suite
cargo r --bin png-test-suite
```

## Reading

http://www.libpng.org/pub/png/pngintro.html<br>
https://www.w3.org/TR/2003/REC-PNG-20031110/<br>
http://www.libpng.org/pub/png/pngpic2.html<br>
https://www.w3.org/Graphics/PNG/platform.html<br>

https://sotrh.github.io/learn-wgpu/<br>
