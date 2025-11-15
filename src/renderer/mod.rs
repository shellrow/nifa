pub mod json;
pub mod table;
pub mod tree;
pub mod yaml;

pub fn fmt_bps(bps: u64) -> String {
    const K: f64 = 1_000.0;
    let b = bps as f64;
    if b >= K * K * K {
        format!("{:.2} Gb/s", b / (K * K * K))
    } else if b >= K * K {
        format!("{:.2} Mb/s", b / (K * K))
    } else if b >= K {
        format!("{:.2} Kb/s", b / K)
    } else {
        format!("{} b/s", bps)
    }
}

pub fn fmt_flags(flags: u32) -> String {
    format!("0x{:08X}", flags)
}
