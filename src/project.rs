use std::{collections::HashMap, path::PathBuf};

use crate::{
    content::content::{Content, GenericContentData},
    curve::{Curve, CurveData},
    video::{Video, VideoType, VideoTypeEnum},
    video_render_settings::{FrameRenderInfo, VideoRenderSettings},
};
use std::sync::{Arc, Mutex};

pub type SharedCurvesId = u64;

#[derive(Clone)]
pub struct Project {
    pub proj: Arc<Mutex<ProjectData>>,
    vid: Option<Arc<Mutex<Video>>>,
    pub shared_curves: SharedCurves,
}
pub struct ProjectData {
    pub name: String,
    pub path: Option<PathBuf>,
    pub render_settings_export: Option<VideoRenderSettings>,
}
impl Default for ProjectData {
    fn default() -> Self {
        Self {
            name: "Unnamed Project".into(),
            path: None, // Some("/tmp/dummi26_rusty_editor_unnamed_project.txt".into()),
            render_settings_export: Some(VideoRenderSettings::export(FrameRenderInfo {
                out_vid_aspect_ratio: 16.0 / 9.0,
            })),
        }
    }
}
impl Project {
    pub fn new(proj: ProjectData) -> Self {
        let mut s = Self {
            proj: Arc::new(Mutex::new(proj)),
            vid: None,
            shared_curves: SharedCurves::new(),
        };
        s.vid = Some(Arc::new(Mutex::new(Video::new_full(VideoType::new(
            VideoTypeEnum::List(vec![]),
            GenericContentData::new(s.clone()),
        )))));
        s.vid().lock().unwrap().generic_content_data().project = s.clone(); // with vid set properly
        s
    }
    pub fn vid(&self) -> Arc<Mutex<Video>> {
        self.vid.as_ref().unwrap().clone()
    }
    pub fn add_vid(&mut self, vid: Arc<Mutex<Video>>) {
        self.vid = Some(vid)
    }
}

#[derive(Clone)]
pub struct SharedCurves {
    curves: Arc<Mutex<HashMap<SharedCurvesId, CurveData>>>,
    free_ids: Arc<Mutex<Vec<SharedCurvesId>>>,
}
impl SharedCurves {
    pub fn new() -> Self {
        Self {
            curves: Arc::new(Mutex::new(HashMap::new())),
            free_ids: Arc::new(Mutex::new(vec![])),
        }
    }
    pub fn get(&self, id: &SharedCurvesId) -> Option<CurveData> {
        match self.curves.lock().unwrap().get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    pub fn insert(&self, curve: CurveData) -> SharedCurvesId {
        let mut curves = self.curves.lock().unwrap();
        let id = if let Some(id) = self.free_ids.lock().unwrap().pop() {
            id
        } else {
            curves.len() as _
        };
        curves.insert(id, curve);
        id
    }
}
