//! #208：内联汇编模板的效应判定（保守启发式）。
//! 纯寄存器算术（leaq/addq/imulq/cmov* 等）不计 state；访存操作数（`(...)`/`[...]`）、
//! 特权/系统/字符串指令以及未知助记符一律计 state（避免 memory(none) 误标，#206）。
pub(super) fn asm_may_touch_memory(template: &str) -> bool {
    const STATE_INSN: &[&str] = &[
        "syscall", "sysenter", "int", "lock", "rep", "repe", "repne",
        "stosb", "stosw", "stosd", "stosq", "movsb", "movsw", "movsd", "movsq",
        "cmpsb", "cmpsw", "cmpsd", "cmpsq", "scasb", "scasw", "scasd", "scasq",
        "lodsb", "lodsw", "lodsd", "lodsq",
        "mfence", "sfence", "lfence", "clflush",
        "xchg", "xchgb", "xchgw", "xchgq", "cmpxchg", "cmpxchgb", "cmpxchgw", "cmpxchgq",
        "cpuid", "rdtsc", "rdtscp", "hlt", "cli", "sti",
        "in", "inb", "inw", "inl", "out", "outb", "outw", "outl",
        "call", "callq", "jmp", "ret", "push", "pushq", "pop", "popq",
    ];
    const PURE_INSN: &[&str] = &[
        "leaq", "lea", "addq", "addl", "adcq", "adcl", "subq", "subl", "sbbq", "sbbl",
        "imulq", "imull", "mulq", "mull", "idivq", "idivl", "divq", "divl",
        "andq", "andl", "orq", "orl", "xorq", "xorl", "notq", "notl",
        "negq", "negl", "shlq", "shll", "shrq", "shrl", "sarq", "sarl",
        "shldq", "shrdq", "rolq", "rorq",
        "cqto", "cltq", "cltd", "movq", "movl", "mov", "movabsq",
        "bswap", "bsfq", "bsrq", "popcntq", "lzcntq", "tzcntq",
        "nop", "ud2",
    ];
    for raw in template.split(['\n', ';']) {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let mnemonic = line.split_whitespace().next().unwrap_or("").trim_end_matches(':');
        let lower = mnemonic.to_ascii_lowercase();
        if STATE_INSN.contains(&lower.as_str()) {
            return true;
        }
        // 访存操作数：除 lea（纯地址计算）外的 `(...)`/`[...]`
        if lower != "leaq" && lower != "lea" && (line.contains('(') || line.contains('[')) {
            return true;
        }
        // setcc 写字节寄存器，纯
        if lower.starts_with("set") || lower.starts_with("cmov") {
            continue;
        }
        if !PURE_INSN.contains(&lower.as_str()) {
            return true;
        }
    }
    false
}
