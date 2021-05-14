use opencv::{features2d, imgcodecs, core};
pub struct SiftParse{
    img1: core::Mat,
    img2: core::Mat,
    detector: core::Ptr<features2d::SIFT>
}

impl SiftParse{
    pub fn create(file1: &str, file2: &str) -> SiftParse{
        let imread_type  = imgcodecs::ImreadModes::IMREAD_COLOR;
        SiftParse{
            detector: features2d::SIFT::create(100, 3, 0.09, 1.0, 1.0).unwrap(),
            img1: imgcodecs::imread(file1, imread_type as i32).unwrap(),
            img2: imgcodecs::imread(file2, imread_type as i32).unwrap(),
        }

    }
}