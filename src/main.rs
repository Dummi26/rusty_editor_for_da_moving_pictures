mod useful;
mod video;
mod video_render_settings;
mod video_cached_frames;
mod input_video;
mod content;
mod effect;
mod curve;
mod project;
mod gui;
mod multithreading;
mod files;

// ffmpeg -i vids/video.mp4 path/%09d.png

fn main() {
    if gui::IS_ENABLED {
        gui::main();
    }
}