use rltk::rex::XpFile;

rltk::embedded_resource!(BACKGROUND, "../resources/background_80x50.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(BACKGROUND, "../resources/background_80x50.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/background_80x50.xp").unwrap(),
        }
    }
}
