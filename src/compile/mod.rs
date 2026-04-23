use std::{
    ffi::{CStr, c_char}, fs::{self, File}, path::{Path, PathBuf}, process::Command
};

use inkwell::{
    OptimizationLevel,
    context::Context,
    memory_buffer::MemoryBuffer,
    module::Module,
    targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
    },
};

use crate::pakager::{Arch, Pakage, System, UseMode};

#[allow(dead_code)]
fn get_arch(arch: Arch) -> &'static str {
    match arch {
        Arch::X86X64 => "x86_64",
        Arch::X86 => "i386",
        Arch::AArch64 => "aarch64",
        Arch::Arm => "arm",
        Arch::RiscV64 => "riscv64",
        Arch::Mips64 => "mips64",
        Arch::PowerPC64 => "powerpc64",
        Arch::S390x => "s390x",
        Arch::Default => "x86_64",
    }
}

#[allow(dead_code)]
fn get_system(system: System) -> Option<&'static str> {
    match system {
        System::Windows => Some("pc-windows-msvc"),
        System::Linux => Some("unknown-linux-gnu"),
        System::Default => None,
    }
}

#[allow(dead_code)]
pub fn compile(pak_path: &str, output_path: &str) {
    if !pak_path.ends_with(".lcl") {
        panic!("pak path must end with .lcl");
    }

    let pak = Pakage::analyze_pak(pak_path);
    let name = Path::new(&pak.info.name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();

    let init_config = InitializationConfig {
        asm_printer: true,
        asm_parser: true,
        base: true,
        disassembler: true,
        info: true,
        machine_code: true,
    };
    Target::initialize_all(&init_config);

    let triple = match get_system(pak.system) {
        None => TargetMachine::get_default_triple(),
        Some(os_suffix) => {
            let triple_str = format!("{}-{}", get_arch(pak.arch), os_suffix);
            TargetTriple::create(&triple_str)
        }
    };
    let target = Target::from_triple(&triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    // 读取 pak 内的 bitcode 并构建模块
    let context = Context::create();
    let buffer = MemoryBuffer::create_from_memory_range_copy(&pak.bc, "pak_bc");
    let module = Module::parse_bitcode_from_buffer(&buffer, &context)
        .expect("failed to parse bitcode from pak");

    // 输出目录：output_path 视为文件夹，不存在则创建
    let out_dir = PathBuf::from(output_path);
    if let Err(e) = fs::create_dir_all(&out_dir) {
        println!("create output dir failed: {}", e);
        return;
    }

    // 将 LLVM IR 写入文件以供检查（开发者模式）
    if pak.use_mode == UseMode::AsDeveloper {
        let lir = module.print_to_string().to_string();
        let lir_path = out_dir.join("lir.txt");
        fs::write(lir_path, &lir).unwrap();
    }

    // 设置 module 的 data layout 与 triple
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
    module.set_triple(&triple);

    // 生成汇编与目标文件
    let output_asm = out_dir.join(format!("{}.s", name));
    if let Err(e) = target_machine.write_to_file(&module, FileType::Assembly, &output_asm) {
        if let Some(parent) = output_asm.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = File::create(&output_asm);
        if let Err(e) = target_machine.write_to_file(&module, FileType::Assembly, &output_asm) {
            println!("write asm failed: {}", e);
            return;
        }
    }

    let output_obj = out_dir.join(format!("{}.o", name));
    if let Err(e) = target_machine.write_to_file(&module, FileType::Object, &output_obj) {
        if let Some(parent) = output_obj.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = File::create(&output_obj);
        if let Err(e) = target_machine.write_to_file(&module, FileType::Object, &output_obj) {
            println!("write obj failed: {}", e);
            return;
        }
    }

    // 按 pak.target 生成最终产物
    let link_map = out_dir.join(format!("{}.link.map", name));

    let link = match pak.target {
        crate::pakager::Target::Executable => Command::new("g++")
            .arg("-g")
            .arg("-O0")
            .arg(output_obj.to_string_lossy().to_string())
            .arg("-L./runtime")
            .arg("-layanami_runtime")
            .arg(format!("-Wl,-Map={}", link_map.to_string_lossy()))
            .arg("-Wl,-t")
            .arg("-o")
            .arg(out_dir.join(&name).to_string_lossy().to_string())
            .status(),
        crate::pakager::Target::StaticLib => Command::new("ar")
            .arg("rcs")
            .arg(
                out_dir
                    .join(format!("lib{}.a", name))
                    .to_string_lossy()
                    .to_string(),
            )
            .arg(output_obj.to_string_lossy().to_string())
            .status(),
        crate::pakager::Target::DynamicLib => Command::new("g++")
            .arg("-shared")
            .arg(output_obj.to_string_lossy().to_string())
            .arg("-L./runtime")
            .arg("-layanami_runtime")
            .arg(format!("-Wl,-Map={}", link_map.to_string_lossy()))
            .arg("-Wl,-t")
            .arg("-o")
            .arg(
                out_dir
                    .join(format!("lib{}.so", name))
                    .to_string_lossy()
                    .to_string(),
            )
            .status(),
    };

    match link {
        Ok(status) => {
            if !status.success() {
                println!("link not success, {:?}", status.code());
            }
        }
        Err(e) => {
            println!("link command failed: {}", e);
            return;
        }
    }

    // 用户模式删除原 pak
    if pak.use_mode == UseMode::AsUser {
        let _ = fs::remove_file(pak_path);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn compile_api(c_pak_path:*const c_char, c_output_path:*const c_char) {
    if c_pak_path.is_null() || c_output_path.is_null() {
        return;
    }

    let pak_path = unsafe { CStr::from_ptr(c_pak_path) };
    let s1 = match pak_path.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };
    let output_path = unsafe { CStr::from_ptr(c_output_path) };
    let s2 = match output_path.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };


    compile(s1, s2);
}