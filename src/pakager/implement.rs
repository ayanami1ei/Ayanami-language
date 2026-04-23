use std::{
    cell::RefCell,
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    rc::Weak,
};

use crate::{
    pakager::{
        Arch, PackageHeader, Pakage, PakageInfo, System, Target, UseMode,
        tlv::{Tlv, TlvType},
    },
    symbol_table::{Scope, Symbol},
    types::VarType,
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

impl Symbol {
    pub(crate) fn get_tlv(&self) -> Vec<Tlv> {
        let mut tlvs = Vec::new();

        // Name TLV
        let name_tlv = Tlv::new(TlvType::SymbolsName, self.name.clone().into_bytes());
        tlvs.push(name_tlv);

        // IsFunc TLV
        let is_func_tlv = Tlv::new(
            TlvType::SymbolIsFunc,
            vec![if self.is_func { 1u8 } else { 0u8 }],
        );
        tlvs.push(is_func_tlv);

        // Args TLV
        let args_tlv = Tlv::new(TlvType::SymbolArgs, Self::serialize_args(&self.args));
        tlvs.push(args_tlv);

        // IsArgc TLV
        let is_argc_tlv = Tlv::new(
            TlvType::SymbolIsArgc,
            vec![if self.is_argc { 1u8 } else { 0u8 }],
        );
        tlvs.push(is_argc_tlv);

        // IsRef TLV
        let is_ref_tlv = Tlv::new(
            TlvType::SymbolIsRef,
            vec![if self.is_ref { 1u8 } else { 0u8 }],
        );
        tlvs.push(is_ref_tlv);

        // IsVar TLV
        let is_var_tlv = Tlv::new(
            TlvType::SymbolIsVar,
            vec![if self.is_var { 1u8 } else { 0u8 }],
        );
        tlvs.push(is_var_tlv);

        // ItsType TLV
        let its_type_tlv = Tlv::new(
            TlvType::SymbolItsType,
            Self::serialize_var_type_set(&self.its_type),
        );
        tlvs.push(its_type_tlv);

        // IsArr TLV
        let is_arr_tlv = Tlv::new(
            TlvType::SymbolIsArr,
            vec![if self.is_arr { 1u8 } else { 0u8 }],
        );
        tlvs.push(is_arr_tlv);

        // ElemType TLV
        let elem_type_tlv = Tlv::new(
            TlvType::SymbolElemType,
            Self::serialize_var_type_sets(&self.elem_type),
        );
        tlvs.push(elem_type_tlv);

        // Id TLV
        let id_tlv = Tlv::new(TlvType::SymbolId, self.id.to_le_bytes().to_vec());
        tlvs.push(id_tlv);

        // Level TLV
        let level_tlv = Tlv::new(TlvType::SymbolLevel, self.level.to_le_bytes().to_vec());
        tlvs.push(level_tlv);

        // ScopeId TLV
        let scope_id_tlv = Tlv::new(TlvType::SymbolScopeId, self.scope_id.to_le_bytes().to_vec());
        tlvs.push(scope_id_tlv);

        // BodyScopeId TLV (optional)
        if let Some(body_scope_id) = self.body_scope_id {
            let body_scope_tlv = Tlv::new(
                TlvType::SymbolBodyScopeId,
                body_scope_id.to_le_bytes().to_vec(),
            );
            tlvs.push(body_scope_tlv);
        }

        tlvs
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for tlv in self.get_tlv() {
            bytes.extend(tlv.as_bytes());
        }
        bytes
    }

    fn serialize_args(args: &Vec<Symbol>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&(args.len() as u32).to_le_bytes());
        for arg in args {
            let arg_bytes = arg.to_bytes();
            bytes.extend(&(arg_bytes.len() as u32).to_le_bytes());
            bytes.extend(arg_bytes);
        }
        bytes
    }

    fn serialize_var_type_set(set: &HashSet<VarType>) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut values: Vec<u32> = set.iter().map(|t| t.clone() as u32).collect();
        values.sort_unstable();
        bytes.extend(&(values.len() as u32).to_le_bytes());
        for v in values {
            bytes.extend(&v.to_le_bytes());
        }
        bytes
    }

    fn serialize_var_type_sets(sets: &Vec<HashSet<VarType>>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&(sets.len() as u32).to_le_bytes());
        for set in sets {
            bytes.extend(Self::serialize_var_type_set(set));
        }
        bytes
    }

    pub(crate) fn from_tlv_bytes(bytes: &[u8]) -> Vec<Symbol> {
        let mut offset = 0usize;
        let mut symbols: Vec<Symbol> = Vec::new();
        let mut current = SymbolData::default();
        let mut has_data = false;

        while let Some((ty, value)) = Self::read_tlv(bytes, &mut offset) {
            if ty == TlvType::SymbolsName as u8 {
                if has_data {
                    if let Some(sym) = current.into_symbol() {
                        symbols.push(sym);
                    }
                    current = SymbolData::default();
                }
                current.name = Some(String::from_utf8(value).unwrap_or_default());
                has_data = true;
                continue;
            }

            has_data = true;
            match ty {
                t if t == TlvType::SymbolIsFunc as u8 => {
                    current.is_func = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolArgs as u8 => {
                    current.args = Self::deserialize_args(&value);
                }
                t if t == TlvType::SymbolIsArgc as u8 => {
                    current.is_argc = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolIsRef as u8 => {
                    current.is_ref = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolIsVar as u8 => {
                    current.is_var = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolItsType as u8 => {
                    current.its_type = Self::deserialize_var_type_set(&value);
                }
                t if t == TlvType::SymbolIsArr as u8 => {
                    current.is_arr = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolElemType as u8 => {
                    current.elem_type = Self::deserialize_var_type_sets(&value);
                }
                t if t == TlvType::SymbolId as u8 => {
                    current.id = Self::read_i32(&value).unwrap_or(0);
                }
                t if t == TlvType::SymbolLevel as u8 => {
                    current.level = Self::read_i32(&value).unwrap_or(0);
                }
                t if t == TlvType::SymbolScopeId as u8 => {
                    current.scope_id = Self::read_i32(&value).unwrap_or(0);
                }
                t if t == TlvType::SymbolBodyScopeId as u8 => {
                    current.body_scope_id = Self::read_i32(&value);
                }
                _ => {}
            }
        }

        if has_data {
            if let Some(sym) = current.into_symbol() {
                symbols.push(sym);
            }
        }

        symbols
    }

    fn read_tlv(bytes: &[u8], offset: &mut usize) -> Option<(u8, Vec<u8>)> {
        if *offset >= bytes.len() {
            return None;
        }
        let ty = bytes[*offset];
        *offset += 1;
        if *offset + 4 > bytes.len() {
            return None;
        }
        let mut len_buf = [0u8; 4];
        len_buf.copy_from_slice(&bytes[*offset..*offset + 4]);
        let len = u32::from_le_bytes(len_buf) as usize;
        *offset += 4;
        if *offset + len > bytes.len() {
            return None;
        }
        let value = bytes[*offset..*offset + len].to_vec();
        *offset += len;
        Some((ty, value))
    }

    fn read_i32(bytes: &[u8]) -> Option<i32> {
        if bytes.len() < 4 {
            return None;
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[..4]);
        Some(i32::from_le_bytes(buf))
    }

    fn deserialize_args(bytes: &[u8]) -> Vec<Symbol> {
        let mut args = Vec::new();
        let mut offset = 0usize;

        let count = Self::read_u32(bytes, &mut offset).unwrap_or(0) as usize;
        for _ in 0..count {
            let len = match Self::read_u32(bytes, &mut offset) {
                Some(v) => v as usize,
                None => break,
            };
            if offset + len > bytes.len() {
                break;
            }
            let arg_bytes = &bytes[offset..offset + len];
            offset += len;
            if let Some(arg) = Self::parse_symbol_from_bytes(arg_bytes) {
                args.push(arg);
            }
        }

        args
    }

    fn parse_symbol_from_bytes(bytes: &[u8]) -> Option<Symbol> {
        let mut offset = 0usize;
        let mut data = SymbolData::default();

        while let Some((ty, value)) = Self::read_tlv(bytes, &mut offset) {
            match ty {
                t if t == TlvType::SymbolsName as u8 => {
                    data.name = Some(String::from_utf8(value).unwrap_or_default());
                }
                t if t == TlvType::SymbolIsFunc as u8 => {
                    data.is_func = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolArgs as u8 => {
                    data.args = Self::deserialize_args(&value);
                }
                t if t == TlvType::SymbolIsArgc as u8 => {
                    data.is_argc = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolIsRef as u8 => {
                    data.is_ref = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolIsVar as u8 => {
                    data.is_var = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolItsType as u8 => {
                    data.its_type = Self::deserialize_var_type_set(&value);
                }
                t if t == TlvType::SymbolIsArr as u8 => {
                    data.is_arr = value.first().copied().unwrap_or(0) != 0;
                }
                t if t == TlvType::SymbolElemType as u8 => {
                    data.elem_type = Self::deserialize_var_type_sets(&value);
                }
                t if t == TlvType::SymbolId as u8 => {
                    data.id = Self::read_i32(&value).unwrap_or(0);
                }
                t if t == TlvType::SymbolLevel as u8 => {
                    data.level = Self::read_i32(&value).unwrap_or(0);
                }
                t if t == TlvType::SymbolScopeId as u8 => {
                    data.scope_id = Self::read_i32(&value).unwrap_or(0);
                }
                t if t == TlvType::SymbolBodyScopeId as u8 => {
                    data.body_scope_id = Self::read_i32(&value);
                }
                _ => {}
            }
        }

        data.into_symbol()
    }

    fn deserialize_var_type_set(bytes: &[u8]) -> HashSet<VarType> {
        let mut offset = 0usize;
        Self::deserialize_var_type_set_at(bytes, &mut offset)
    }

    fn deserialize_var_type_set_at(bytes: &[u8], offset: &mut usize) -> HashSet<VarType> {
        let mut set = HashSet::new();
        let count = Self::read_u32(bytes, offset).unwrap_or(0) as usize;
        for _ in 0..count {
            let v = match Self::read_u32(bytes, offset) {
                Some(v) => v,
                None => break,
            };
            set.insert(Self::var_type_from_u32(v));
        }
        set
    }

    fn deserialize_var_type_sets(bytes: &[u8]) -> Vec<HashSet<VarType>> {
        let mut sets = Vec::new();
        let mut offset = 0usize;
        let count = Self::read_u32(bytes, &mut offset).unwrap_or(0) as usize;
        for _ in 0..count {
            let set = Self::deserialize_var_type_set_at(bytes, &mut offset);
            sets.push(set);
        }
        sets
    }

    fn var_type_from_u32(v: u32) -> VarType {
        match v {
            1 => VarType::Int,
            2 => VarType::Float,
            3 => VarType::Bool,
            4 => VarType::Char,
            5 => VarType::String,
            6 => VarType::Array,
            7 => VarType::Unknown,
            _ => VarType::Unknown,
        }
    }

    fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
        if *offset + 4 > bytes.len() {
            return None;
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[*offset..*offset + 4]);
        *offset += 4;
        Some(u32::from_le_bytes(buf))
    }
}

#[derive(Default)]
struct SymbolData {
    name: Option<String>,
    is_func: bool,
    args: Vec<Symbol>,
    is_argc: bool,
    is_ref: bool,
    is_var: bool,
    its_type: HashSet<VarType>,
    is_arr: bool,
    elem_type: Vec<HashSet<VarType>>,
    id: i32,
    level: i32,
    scope_id: i32,
    body_scope_id: Option<i32>,
}

impl SymbolData {
    fn into_symbol(self) -> Option<Symbol> {
        let name = self.name?;
        Some(Symbol {
            name,
            is_func: self.is_func,
            args: self.args,
            is_argc: self.is_argc,
            is_ref: self.is_ref,
            is_var: self.is_var,
            its_type: self.its_type,
            id: self.id,
            level: self.level,
            scope_id: self.scope_id,
            area: Weak::<RefCell<Scope>>::new(),
            body_scope_id: self.body_scope_id,
            is_arr: self.is_arr,
            elem_type: self.elem_type,
        })
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
        symbols: Vec<Symbol>,
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
            symbols,
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

        let mut symbols_bytes = Vec::new();
        for symbol in &self.symbols {
            symbols_bytes.extend(symbol.to_bytes());
        }
        pak_bytes.push(Tlv::new(TlvType::Symbol, symbols_bytes));

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
                0x0A => TlvType::Symbol,
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
        let mut symbols = Vec::new();
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
                TlvType::Symbol => {
                    symbols = Symbol::from_tlv_bytes(&i.value);
                }
                TlvType::Bc => {
                    bc = i.value;
                }
                _=> panic!("Unexpect TLV type in package"),
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
            symbols,
            bc,
        }
    }
}
