pub(crate) mod implements;

pub(crate) struct Pakager{

}

struct PackageHeader {
    magic:String,        // 文件标识
    version:u16,         // 格式版本
    flags:u16,           // 特性位
    header_size:u32,     // header 总大小
    toc_offset:u32,      // 目录表偏移
    toc_count:u32,       // 目录项数量
    file_size:u64,       // 文件总大小
}
