# Parser v0 doc

# Video

pos:012345

0: alignment (what part of the video the position values point to; one char):

    1^2 | top left, top, top right
    <+> | left, center, right
    3v4 | bottom left, bottom, bottom right
    ! | custom - must be followed by '[x][y]' (x and y are curves)

1, 2, 3, 4: x, y, width, height (curves)

## Vid from frames (VidFromImagesInDirectory)

\[path]\\\[first frame]\[- or +]\[-: last frame (exclusive) | +: how many frames to cut off from the end]

default: \[path]\\0+0; (don't cut off any frames on either side)

## Vid from file using ffmpeg (VidUsingFfmpeg)

\[path]\\

This is very slow and does not cache the video. It uses ffprobe to get the video's length, then ffmpeg to get a frame from the video, write that to /tmp/..., read that and then display the frame.

# Image

[image path];

[image path]<[command];

[image path]<[command]+[arg];

[image path]<[command]+[arg1]+[arg2]+[arg..];
