# Features

To follow along, run the following:

```sh
cd png && cargo r --release nick-diaz.png
```

## Color tones

### Grayscale

Press <kbd>G</kbd> to grayscale.

<img src="grayscale.png" alt="grayscale" width="320">

### Invert

Press <kbd>I</kbd> to invert colors.

<img src="invert.png" alt="invert" width="320">

## Convolution Filters

### Blur

Press <kbd>B</kbd> to blur. It applies a Gaussian blur. Use the <kbd>ArrowUp</kbd> and <kbd>ArrowDown</kbd> to control
the blur radius. Defaults to 21.

<img src="blur.png" alt="gaussian-blur" width="320">

### Sharpen

Press <kbd>S</kbd> to sharpen. It uses a Laplacian-based sharpening filter. Use the <kbd>ArrowUp</kbd> and <kbd>
ArrowDown</kbd> to control the sharpening factor. Defaults to 16.

<img src="sharpen.png" alt="laplacian-sharpen" width="320">

### Detect edges

Press <kbd>E</kbd> to enhance the edges in an image. It applies a Sobel filter.

<img src="edge-detect.png" alt="sobel-edge-detect" width="320">