use std::{
    fs::File,
    io::{Read, Write},
};

use crate::pakager::{
    Arch, PackageHeader, Pakage, PakageInfo, System, Target, UseMode,
    tlv::{Tlv, TlvType},
};

impl PackageHeader {
    pub(crate) fn new() -> Self {
        PackageHeader {
            magic: "笑えばいいと思うよ".to_string(),
            version: "0.0.1".to_string(),
            flags: 0,
        }
    }

    pub(crate) fn get_tlv(&self) -> Vec<Tlv> {
        let mut tlvs = Vec::new();

        // Magic TLV
        let magic_tlv = Tlv::new(TlvType::Magic, self.magic.clone().into_bytes());
        tlvs.push(magic_tlv);

        // Version TLV
        let version_tlv = Tlv::new(TlvType::PakagerVersion, self.version.clone().into_bytes());
        tlvs.push(version_tlv);

        // Flags TLV
        let flags_tlv = Tlv::new(TlvType::Flags, self.flags.to_le_bytes().to_vec());
        tlvs.push(flags_tlv);

        tlvs
    }
}

impl PakageInfo {
    pub(crate) fn get_tlv(&self) -> Vec<Tlv> {
        let mut tlvs = Vec::new();

        // Name TLV
        let name_tlv = Tlv::new(TlvType::Name, self.name.clone().into_bytes());
        tlvs.push(name_tlv);

        // Version TLV
        let version_tlv = Tlv::new(TlvType::Version, self.version.clone().into_bytes());
        tlvs.push(version_tlv);

        tlvs
    }
}

impl UseMode {
    pub(crate) fn get_tlv(&self) -> Tlv {
        let value = match self {
            UseMode::AsDeveloper => 0u8,
            UseMode::AsUser => 1u8,
        };
        Tlv::new(TlvType::UseMode, vec![value])
    }
}

impl Target {
    pub(crate) fn get_tlv(&self) -> Tlv {
        let value = match self {
            Target::Executable => 0u8,
            Target::StaticLib => 1u8,
            Target::DynamicLib => 2u8,
        };
        Tlv::new(TlvType::Target, vec![value])
    }
}

impl Arch {
    pub(crate) fn get_tlv(&self) -> Tlv {
        let value = match self {
            Arch::X86X64 => 0u8,
            Arch::X86 => 1u8,
            Arch::AArch64 => 2u8,
            Arch::Arm => 3u8,
            Arch::RiscV64 => 4u8,
            Arch::Mips64 => 5u8,
            Arch::PowerPC64 => 6u8,
            Arch::S390x => 7u8,
            Arch::Default => 255u8,
        };
        Tlv::new(TlvType::Arch, vec![value])
    }
}

impl System {
    pub(crate) fn get_tlv(&self) -> Tlv {
        let value = match self {
            System::Windows => 0u8,
            System::Linux => 1u8,
            System::Default => 255u8,
        };
        Tlv::new(TlvType::System, vec![value])
    }
}

impl Pakage {
    pub(crate) fn new(
        name: String,
        version: String,
        use_mode: UseMode,
        target: Target,
        arch: Arch,
        system: System,
        bc_path: String,
    ) -> Self {
        let bc = std::fs::read(bc_path).expect("Failed to read bytecode file");
        Pakage {
            header: PackageHeader::new(),
            info: PakageInfo { name, version },
            use_mode,
            target,
            arch,
            system,
            bc,
        }
    }

    pub(crate) fn gen_pak(&self, path: &str) {
        let mut pak_bytes = Vec::new();

        // Serialize header TLVs
        pak_bytes.append(&mut self.header.get_tlv());

        // Serialize info TLVs
        pak_bytes.append(&mut self.info.get_tlv());

        // Serialize use mode TLV
        pak_bytes.push(self.use_mode.get_tlv());
        // Serialize target TLV
        pak_bytes.push(self.target.get_tlv());

        // Serialize arch TLV
        pak_bytes.push(self.arch.get_tlv());

        // Serialize system TLV
        pak_bytes.push(self.system.get_tlv());

        // Append bytecode
        pak_bytes.push(Tlv::new(TlvType::Bc, self.bc.clone()));

        let mut bytes = Vec::new();
        for i in pak_bytes {
            bytes.extend(i.as_bytes());
        }

        // Write to file
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut pakage = File::create(path).expect("Failed to create package file");
        pakage
            .write_all(&bytes)
            .expect("Failed to write package file");
    }

    #[allow(dead_code)]
    pub(crate) fn analyze_pak(path: &str) -> Pakage {
        let pak = File::open(path).expect("Failed to open package file");
        let mut tlvs = Vec::new();
        let mut reader = std::io::BufReader::new(pak);

        loop {
            let mut type_buf = [0u8; 1];
            if reader.read_exact(&mut type_buf).is_err() {
                break;
            }
            let tlv_type = match type_buf[0] {
                0x00 => TlvType::Magic,
                0x01 => TlvType::PakagerVersion,
                0x02 => TlvType::Flags,
                0x03 => TlvType::Name,
                0x04 => TlvType::Version,
                0x05 => TlvType::UseMode,
                0x06 => TlvType::Target,
                0x07 => TlvType::Arch,
                0x08 => TlvType::Bc,
                0x09 => TlvType::System,
                _ => panic!("Unknown TLV type"),
            };

            let mut length_buf = [0u8; 4];
            reader
                .read_exact(&mut length_buf)
                .expect("Failed to read TLV length");
            let length = u32::from_le_bytes(length_buf) as usize;

            let mut value_buf = vec![0u8; length];
            reader
                .read_exact(&mut value_buf)
                .expect("Failed to read TLV value");

            tlvs.push(Tlv::new(tlv_type, value_buf));
        }

        let mut header = PackageHeader::new();
        let mut info = PakageInfo {
            name: String::new(),
            version: String::new(),
        };
        let mut use_mode = UseMode::default();
        let mut target = Target::default();
        let mut arch = Arch::default();
        let mut system = System::default();
        let mut bc = Vec::new();

        for i in tlvs {
            match i.ty {
                TlvType::Magic => {
                    header.magic = String::from_utf8(i.value).expect("Invalid UTF-8 in magic");
                }
                TlvType::PakagerVersion => {
                    header.version = String::from_utf8(i.value).expect("Invalid UTF-8 in version");
                }
                TlvType::Flags => {
                    let mut arr = [0u8; 2];
                    arr.copy_from_slice(&i.value[..2]);
                    header.flags = u16::from_le_bytes(arr);
                }
                TlvType::Name => {
                    info.name = String::from_utf8(i.value).expect("Invalid UTF-8 in name");
                }
                TlvType::Version => {
                    info.version = String::from_utf8(i.value).expect("Invalid UTF-8 in version");
                }
                TlvType::UseMode => match i.value[0] {
                    0 => use_mode = UseMode::AsDeveloper,
                    1 => use_mode = UseMode::AsUser,
                    _ => panic!("Unknown UseMode value"),
                },
                TlvType::Target => match i.value[0] {
                    0 => target = Target::Executable,
                    1 => target = Target::StaticLib,
                    2 => target = Target::DynamicLib,
                    _ => panic!("Unknown Target value"),
                },
                TlvType::Arch => match i.value[0] {
                    0 => arch = Arch::X86X64,
                    1 => arch = Arch::X86,
                    2 => arch = Arch::AArch64,
                    3 => arch = Arch::Arm,
                    4 => arch = Arch::RiscV64,
                    5 => arch = Arch::Mips64,
                    6 => arch = Arch::PowerPC64,
                    7 => arch = Arch::S390x,
                    254 | 255 => arch = Arch::Default,
                    _ => panic!("Unknown Arch value"),
                },
                TlvType::System => match i.value[0] {
                    0 => system = System::Windows,
                    1 => system = System::Linux,
                    254 | 255 => system = System::Default,
                    _ => panic!("Unknown System value"),
                },
                TlvType::Bc => {
                    bc = i.value;
                }
            }
        }

        if header.magic != "笑えばいいと思うよ" {
            panic!("Invalid package magic");
        }

        Pakage {
            header,
            info,
            use_mode,
            target,
            arch,
            system,
            bc,
        }
    }
}

/*fn asm(){
    let init_config = InitializationConfig {
            asm_printer: true,
            asm_parser: true,
            base: true,
            disassembler: true,
            info: true,
            machine_code: true,
        };
        Target::initialize_all(&init_config);

        let triple = &TargetMachine::get_default_triple();
        let target = Target::from_triple(triple).unwrap();
        let target_machine = target
            .create_target_machine(
                triple,
                "generic",
                "",
                OptimizationLevel::Aggressive,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();

        // 将 LLVM IR 写入文件以供检查
        let lir = self.module.print_to_string().to_string();
        fs::write("./build/lir.txt", &lir).unwrap();

        // 先尝试生成汇编文件（避免直接生成 object 以确定崩溃是否与后端有关）
        let output_asm = Path::new("./build/test.s");

        // 设置 module 的 data layout 与 triple，保证 TargetMachine 能正确生成目标文件
        self.module
            .set_data_layout(&target_machine.get_target_data().get_data_layout());
        self.module.set_triple(triple);

        // 写入汇编（如果此处也崩溃则问题更靠前）
        target_machine
            .write_to_file(&self.module, FileType::Assembly, output_asm)
            .unwrap();

        let output_obj = Path::new("./build/test.o");
        target_machine
            .write_to_file(&self.module, FileType::Object, output_obj)
            .unwrap();

        let link = Command::new("g++")
            .arg("-g")
            .arg("-O0")
            .arg("./build/test.o")
            .arg("-L./runtime")
            .arg("-layanami_runtime")
            .arg("-o")
            .arg("./build/ayanami_test")
            .status()
            .unwrap();

        if !link.success() {
            println!("g++ not success, {:?}", link.code());
        }
}*/
