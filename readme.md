# Rusty Editor

A video editor that focuses on options, with the goal of letting you do pretty much anything.

## Stand-out features

### Curves

Most numerical properties are curves. This achieves similar things as breakpoints do in other editors (i.e. smoothly zooming or moving things around on screen), but is more flexible.
Curves are separated into independant segments, and each curve segment has a certain type: A constant number, a linear function, and (in the future) multiple options for smooth curves as well.

### Containers

A "Video" is anything that can be displayed on screen. This obviously includes a normal video file, but also an image, some text, or a list of even more videos.

A video has certain properties such as its position, its size, when it starts and for how long it should be on screen. These properties are relative to the video's container.

## Demo

### Containers

The images (top half of the video preview) are both part of the List. Their size changes with the list's size (which changes smoothly from 0 to 100%), while the Images both have a constant width of 50% (represented by a horizontal line in the curve editor). The "Shake" effect applied to the bottom image is not a property of the video, the effect *contains* the image (represented by the indentation to the right in the project tree).

https://user-images.githubusercontent.com/67615357/199796525-d81f11ce-b213-4a0b-804c-79725c6138e7.mp4

### GUI

https://user-images.githubusercontent.com/67615357/199803303-e50295f0-25a3-4acd-b5b3-21afa3f1a305.mp4

(This is still very experimental)

Ctrl+Space: QVidRunner (command bar, also happens to be useful for deleting stuff from the gui, although this is technically a bug)

Esc: GUI layout editing mode (this is still very buggy - left click and hold to resize splits, right click to change vertical/horizontal)

# Disclaimer

This project is nowhere near finished. Nothing is final, the GUI isn't really usable yet, projects can be loaded from .txt files (src/files/parser_general.rs), but they can't be saved yet, videos can't be loaded (only a directory of image files represening the video's frames) and there are a lot of bugs or just weird/unexpected behavior.

# Bugs

- ESC key is treated like text

- Keyboard focus doesn't exist yet

- Sometimes changes will not be properly shown on the video preview. To fix this, change the preview's size to two other resolutions, then switch back to the one you like. (The preview cache stores frames for at most two resolutions, so changing your resolution 3 times ensures all frames you see are actually new enough to reflect your changes (it stores 2 resolutions because QVidRunner routinely scales the preview window down a little))

- the ui edit mode in general

- editing most things from gui doesn't work yet (most importantly: a curve editor is missing, so all videos have pos [0,0 1x1], but also effects and some other stuff)

- also see src/todo.txt

# Is this useful?

Probably not, but I felt limited by some of the video editors I have used in the past and thought this would be an interesting project no matter the outcome. And since quite a bit of effort has gone into this now, I felt i should also put it on my github.
