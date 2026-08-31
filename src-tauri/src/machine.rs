//! What this machine has, where it can be established honestly.
//!
//! One question — how much memory is installed — asked in one place. It was
//! asked in two, with a copy of the same `sysctl` call in `provider` and in
//! `processing`, which is how the two came to disagree about what to do when the
//! answer is unknown.
//!
//! ## Why the answer is allowed to be nothing
//!
//! Both callers use this to avoid promising work the machine cannot finish: the
//! model picker will not recommend something that cannot fit, and the context
//! window is sized against what is actually there. Both are safe with no answer
//! and fall back to a conservative constant. Neither is safe with a wrong one —
//! a guess that is too high recommends a model that swaps until somebody gives
//! up. So an unknown machine says nothing rather than estimating.

/// Installed physical memory in bytes, or nothing where it cannot be read.
pub(crate) fn memory_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        crate::process::command("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|text| text.trim().parse().ok())
    }
    #[cfg(target_os = "linux")]
    {
        // A file rather than a command: no process to spawn, and the format has
        // been stable for as long as the file has existed.
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|text| parse_meminfo(&text))
    }
    // Windows reports this through GlobalMemoryStatusEx, which means unsafe FFI or
    // a crate. This asks a command instead, the way the macOS branch above asks
    // `sysctl` — because the objection that kept this returning nothing was to
    // *unverified unsafe code*, and a command that cannot be run or cannot be
    // parsed simply answers nothing, which every caller already handles.
    //
    // `wmic` first because it is fast, PowerShell second because `wmic` was
    // removed from Windows 11 24H2. Neither has been run by this project on a
    // Windows machine; what has been tested is the parser, on any machine, which
    // is the same arrangement `/proc/meminfo` already has.
    #[cfg(target_os = "windows")]
    {
        ["wmic", "powershell"].iter().find_map(|tool| {
            let output = match *tool {
                "wmic" => crate::process::command("wmic")
                    .args(["ComputerSystem", "get", "TotalPhysicalMemory"])
                    .output(),
                _ => crate::process::command("powershell")
                    .args([
                        "-NoProfile",
                        "-Command",
                        "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
                    ])
                    .output(),
            };
            output
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .and_then(|text| parse_total_physical_memory(&text))
        })
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

/// The number out of what Windows prints for total physical memory.
///
/// Its own function so it can be tested anywhere rather than only on the platform
/// that produces it — the same arrangement `parse_meminfo` has, and for the same
/// reason: this project has no Windows machine, so the part that can be checked
/// on any machine is the part worth having.
///
/// `wmic` prints a header line and then the value; PowerShell prints the value
/// alone. Taking the first line that is entirely digits reads both, and reads
/// nothing out of an error message, which is what a missing command produces.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn parse_total_physical_memory(text: &str) -> Option<u64> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && line.chars().all(|glyph| glyph.is_ascii_digit()))
        .and_then(|line| line.parse().ok())
        .filter(|bytes| *bytes > 0)
}

/// Installed memory in whole gigabytes, or nothing.
pub(crate) fn memory_gb() -> Option<u32> {
    let bytes = memory_bytes()?;
    u32::try_from(bytes / (1024 * 1024 * 1024))
        .ok()
        .filter(|gb| *gb > 0)
}

/// `MemTotal` out of the text of `/proc/meminfo`.
///
/// Its own function so it can be tested on any machine rather than only on the
/// one platform that has the file. The value is in kibibytes — the unit is in
/// the line and is always `kB`, which the kernel writes to mean 1024 bytes.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_meminfo(text: &str) -> Option<u64> {
    text.lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|kibibytes| kibibytes * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "MemTotal:       16333852 kB\n\
                          MemFree:         1234567 kB\n\
                          MemAvailable:    8765432 kB\n";

    #[test]
    fn mem_total_is_read_as_kibibytes() {
        // 16333852 KiB is the 16 GB a machine of that size actually reports.
        assert_eq!(parse_meminfo(SAMPLE), Some(16_333_852 * 1024));
        assert_eq!(parse_meminfo(SAMPLE).unwrap() / (1024 * 1024 * 1024), 15);
    }

    #[test]
    fn a_file_without_the_line_answers_nothing_rather_than_zero() {
        assert_eq!(parse_meminfo("MemFree: 100 kB\n"), None);
        assert_eq!(parse_meminfo(""), None);
        // Not a number, which is what a truncated read looks like.
        assert_eq!(parse_meminfo("MemTotal:       kB\n"), None);
    }

    #[test]
    fn the_first_matching_line_wins_and_nothing_after_it_matters() {
        let text = "MemTotal:       8000000 kB\nMemTotal:       1 kB\n";
        assert_eq!(parse_meminfo(text), Some(8_000_000 * 1024));
    }

    /// What `wmic` prints: a header, the value, and trailing blank lines.
    #[test]
    fn the_windows_value_is_read_past_its_header() {
        let wmic = "TotalPhysicalMemory  \r\n17179869184          \r\n\r\n";
        assert_eq!(parse_total_physical_memory(wmic), Some(17_179_869_184));
        // PowerShell prints the number alone.
        assert_eq!(
            parse_total_physical_memory("17179869184\r\n"),
            Some(17_179_869_184)
        );
    }

    /// A missing command prints an error, not a number, and an error must not be
    /// read as a machine with some quantity of memory.
    #[test]
    fn an_error_message_is_not_a_quantity() {
        assert_eq!(parse_total_physical_memory(""), None);
        assert_eq!(
            parse_total_physical_memory("'wmic' is not recognized as an internal command"),
            None
        );
        assert_eq!(
            parse_total_physical_memory("TotalPhysicalMemory\r\n\r\n"),
            None
        );
        // Zero is not an answer either.
        assert_eq!(parse_total_physical_memory("0\r\n"), None);
    }

    /// Whatever this machine is, the two answers describe the same number.
    #[test]
    fn gigabytes_agree_with_bytes() {
        match (memory_bytes(), memory_gb()) {
            (Some(bytes), Some(gb)) => {
                assert_eq!(u64::from(gb), bytes / (1024 * 1024 * 1024));
            }
            (Some(bytes), None) => assert!(bytes < 1024 * 1024 * 1024),
            (None, gb) => assert!(gb.is_none()),
        }
    }
}
