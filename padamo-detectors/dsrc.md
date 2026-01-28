# DSRC: detector source

Keywords are case-insensitive. In this tutorial:

```
keyword <parameter:type> [optional]
```

## Base
Floating point value (F): value with decimal point. Exponential writing is supported.

Index (I): Positive integer. 

Multiindex (MI): list of indices. `[I, ...]`

Gridpoint (GP): Multiindex with two values. `[I,I]`

Point (P): pair of floating point values. `(F, F)`

## Shapes
Square
```dsrc
square <bottom_left:P> <size:F>
square <size:F>
```

Rectangle
```dsrc
rect <bottom_left:P> size <size:P>
rect <bottom_left:P> top_right <top_right:P>
```

Hexagon
```dsrc
hexagon <size:F>
```

Abstract polygon. 
```dsrc
polygon <points:space separated list of P>
```

## Pixels
Pixelable:
```dsrc
pixel|grids|append|pixel_offsets_and_rotates|pixel_append_prepend
```

Basic pixel declaration:
```
pixel <index:MI> <pixel_shape:shape>
```

Pixel grid:
```dsrc
grid <lower:GP> <upper:GP> <step:GP> <direction_X:P> <direction_Y:P> <pattern:pixelable>
ndgrid <lower:GP> <upper:GP> <step:GP> <index_axes:GP> <direction_X:P> <direction_Y:P> <pattern:pixelable>
```

Prepend and append:
```dsrc
prepend <indices:MI> <pattern:pixelable>
append <indices:MI> <pattern:pixelable>
```

## Transformations
```dsrc
rotate <angle:F> [deg] <operand:shape|pixelable>
move <offset:P> <operand:shape|pixelable>
```

## Top level declarations:
Base info:
```dsrc
name "DetectorName"
shape <compat_shape:MI>
```
