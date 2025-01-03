# png

This project aims to provide a PNG decoder with SIMD-accelerated filters and a GPU-based renderer.

## Render 

To render an image, simply run `cargo run <image_path>`. For example:

```bash
cargo run ./potatoe.png
```

Currently, the renderer supports image resizing. 

## Profile

To profile `png`, the `profile.sh` script serves as syntatic sugar for running samply's profiling.

```bash
./profile.sh ./reagan.png
```

## Reading

https://www.w3.org/TR/2003/REC-PNG-20031110/<br>
http://www.schaik.com/pngsuite/pngsuite.html<br>
https://www.youtube.com/watch?v=EFUYNoFRHQI<br>
