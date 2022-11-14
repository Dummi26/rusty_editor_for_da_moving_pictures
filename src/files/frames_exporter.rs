use crate::cli::Clz;
use image::DynamicImage;

use crate::project::Project;

pub fn export_to_dir(proj: &Project, settings: &crate::video_export_settings::VideoExportSettings) {
    let vid = &proj.vid;
    let mut vid = vid.lock().unwrap();
    let mut pprogress_percent = u32::MAX;
    for frame in 0..settings.frames {
        // TODO: This never reaches 1.0. Currently, this is desired, but that might change in the future - that is why there is a TODO here.
        let progress = frame as f64 / settings.frames as f64;
        let progress_percent = (progress * 100.0).round() as u32;
        if progress_percent != pprogress_percent {
            pprogress_percent = progress_percent;
            eprintln!("{:03}% done.", progress_percent);
        }
        if let Some(prep_data) = vid.prep_draw(progress) {
            let mut img = DynamicImage::new_rgba8(settings.width, settings.height);
            vid.draw(&mut img, prep_data, match &mut proj.proj.lock().unwrap().render_settings_export {
                Some(v) => v,
                None => panic!("\n{}\n",
                    Clz::error_info("The project you are trying to export does not specify any export settings. Please configure the project's export configuration and try again."),
                ),
            });
            let path = { /* 10 long (u32 max length) */ let mut p = settings.output_path.clone(); p.push(format!("{:010}.png", frame)); p };
            if let Err(err) = img.save(&path) {
                panic!("\n{}{}{}\n{}{}\n",
                    Clz::error_info("Error saving image file to path '"), Clz::undecided(path.to_string_lossy().as_ref()), Clz::error_info("'."),
                    Clz::error_info("Error: "), Clz::error_details(err.to_string().as_str()),
                );
            }
        }
    }
}
