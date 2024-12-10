# png

A PNG parser and renderer. The goals of this project is to have robust parsing and SIMD-accelerated filtering.


## Usage

To render an image, simply run `cargo run <image_path>`. For example:
```bash
cargo run ./tests/potatoe.png
```

<br>

To profile `png`, the `profile.sh` script serves as syntatic sugar for running samply's profiling. 
```bash
./profile.sh ./tests/reagan.png
```

## Reading

https://www.w3.org/TR/2003/REC-PNG-20031110/<br>
http://www.schaik.com/pngsuite/pngsuite.html<br>
https://www.youtube.com/watch?v=EFUYNoFRHQI<br>