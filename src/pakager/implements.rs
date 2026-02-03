use crate::pakager::PackageHeader;

impl PackageHeader{
    pub fn new() -> Self {
        PackageHeader {
            magic: "笑えばいいと思うよ".to_string(),
            version: 1,
            flags: 0,
            header_size: 32,
            toc_offset: 32,
            toc_count: 0,
            file_size: 0,
        }
    }
}