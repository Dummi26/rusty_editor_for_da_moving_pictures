# Parser v0 doc

# Video

pos:012345

0: alignment (what part of the video the position values point to; one char):

    1^2 | top left, top, top right
    <+> | left, center, right
    3v4 | bottom left, bottom, bottom right
    ! | custom - must be followed by '[x][y]' (x and y are curves)

1, 2, 3, 4: x, y, width, height (curves)

## Vid from frames

\[path]\\\[first frame]\[- or +]\[-: last frame (exclusive) | +: how many frames to cut off from the end]

default: \[path]\\0+0; (don't cut off any frames on either side)