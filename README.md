# png

This project aims to provide a PNG decoder with SIMD-accelerated filters and a feature-rich renderer.

## Usage

To render an image, simply run `cargo run <image_path>`. For example:

```bash
cargo run ./tests/potatoe.png
```

## Profile

To profile `png`, the `profile.sh` script serves as syntatic sugar for running samply's profiling.

```bash
./profile.sh ./tests/reagan.png
```

## Reading

https://www.w3.org/TR/2003/REC-PNG-20031110/<br>
http://www.schaik.com/pngsuite/pngsuite.html<br>
https://www.youtube.com/watch?v=EFUYNoFRHQI<br>
