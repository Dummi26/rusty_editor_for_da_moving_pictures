use crate::cli::Clz;

mod useful;
mod cli;
mod video;
mod video_render_settings;
mod video_export_settings;
mod content;
mod effect;
mod curve;
mod project;
mod gui;
mod multithreading;
mod files;
mod assets;

// ffmpeg -i vids/video.mp4 path/%09d.png

// ffmpeg -framerate 30 -pattern_type glob -i '*.png' out.mp4

fn main() {
    let mut args = cli::CustomArgs::read_from_env();
    //
    loop {
        args = match args.action {
            Some(action) => {
                args.action = Some(cli::Action::Exit);
                match action {
                    cli::Action::OpenProjectInGui => gui::main(args),
                    cli::Action::ExportProjectToFrames => export_to_frames(args),
                    cli::Action::Exit => break,
                }
            },
            None => panic!("\n{}\n",
                Clz::error_info("No action was specified! Please use --action [action] to specify one."),
            ),
        };
    };
}

fn export_to_frames(args: cli::CustomArgs) -> cli::CustomArgs {
    eprintln!("{}\n{}",
        Clz::starting("Starting export..."),
        Clz::starting(" [1] Loading project."),
    );
    let path = match &args.project_path {
        Some(v) => v.clone(),
        None => panic!("\n{}\n",
            Clz::error_info("Could not export because no project was specified. Please use --proj-path to point to a project file you would like to export."),
        ),
    };
    let settings = match &args.export_options {
        Some(v) => v,
        None => panic!("\n{}\n",
            Clz::error_info("Could not export because export options were not specified. Please use --export-options to set all required options."),
        ),
    };
    let proj = match files::file_handler::read_from_file(&path) {
        Err(err) => panic!("\n{}\n{}\n",
            Clz::error_info("Encountered an IO error trying to open your project to prepare for export:"),
            Clz::error_details(err.to_string().as_str()),
        ),
        Ok(Err(err)) => panic!("\n{}\n{}\n",
            Clz::error_info("Could not open your project to prepare for export, as the parser returned the following error:"),
            Clz::error_details(err.to_string().as_str()),
        ),
        Ok(Ok(v)) => v,
    };
    eprintln!("{}\n{}",
        Clz::completed(" [1] Loaded project."),
        Clz::starting(" [2] Starting export."),
    );
    files::frames_exporter::export_to_dir(&proj, settings);
    println!("{}\n{}",
        Clz::completed(" [2] Export finished."),
        Clz::completed_info("    If you want to create a video from these frames, open a terminal in the directory with the exported images\n    and run 'ffmpeg -framerate 30 -pattern_type glob -i '*.png' out.mp4' to create a video file from the frames."),
    );
    args
}