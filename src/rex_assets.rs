use rltk::rex::XpFile;

rltk::embedded_resource!(BACKGROUND, "../resources/background_80x50.xp");
rltk::embedded_resource!(INTRO, "../resources/halberd.xp");
rltk::embedded_resource!(ENDING, "../resources/ending.xp");

pub struct RexAssets {
    pub menu: XpFile,
    pub intro: XpFile,
    pub ending: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(BACKGROUND, "../resources/background_80x50.xp");
        rltk::link_resource!(INTRO, "../resources/halberd.xp");
        rltk::link_resource!(ENDING, "../resources/ending.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/background_80x50.xp").unwrap(),
            intro: XpFile::from_resource("../resources/halberd.xp").unwrap(),
            ending: XpFile::from_resource("../resources/ending.xp").unwrap(),
        }
    }
}
