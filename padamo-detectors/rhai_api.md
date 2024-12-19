# Pixel

```rhai
// Valid indices
let index = [1,4];

// Valid vertices
let vertices = [[0.0,0.0],[1.0,0.0],[0.0,1.0]];

// Any coordinate is a floating point value.

// These functions return pixel
new_pixel(index, vertices);
rectangle(index, start_x, start_y, size_x, size_y);
rectangle_centered(index, center_x, center_y, size_x, size_y);
square(index, center_x, center_y, size);

```

# Detector

```rhai
let detector = new_detector(); //Makes new detector
detector.clear(); //Clear all pixels
detector.set_name("VTL"); // Set detector name.
detector.set_shape([16,16]); // Set detector compat shape.

let pixel = square([0,0], 0.0, 0.0, 1.0);
detector.add_pixel(pixel); // Add new pixel

```
