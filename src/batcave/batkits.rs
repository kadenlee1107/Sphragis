// Sphragis — BatKits: Pre-built Tool Bundles by Mission Type
//
// When creating a BatCave, pick a kit to auto-install all tools for that mission.
// Usage: batcave create mylab --kit recon
//
// Kits mirror Kali Linux tool categories:
//   recon        — Reconnaissance & information gathering
//   vuln         — Vulnerability analysis & scanning
//   wireless     — Wireless network attacks
//   webapp       — Web application analysis
//   database     — Database assessment
//   passwords    — Password attacks & cracking
//   exploit      — Exploitation tools
//   sniff        — Sniffing & spoofing
//   postexploit  — Post-exploitation
//   forensics    — Digital forensics
//   reporting    — Reporting & documentation

use crate::batcave::cave;
use crate::drivers::uart;

/// A kit definition: name + list of tools + recommended capabilities.
pub struct Kit {
    pub name: &'static str,
    pub description: &'static str,
    pub tools: &'static [&'static str],
    pub caps: &'static [&'static str], // auto-granted capabilities
}

pub const KITS: &[Kit] = &[
    Kit {
        name: "recon",
        description: "Reconnaissance & information gathering",
        tools: &[
            "nmap", "whois", "dig", "host", "traceroute", "ping",
            "netstat", "arp", "ifconfig", "ip", "nslookup",
            "wget", "nc", "hostname", "env", "uname",
        ],
        caps: &["net"],
    },
    Kit {
        name: "vuln",
        description: "Vulnerability analysis & scanning",
        tools: &[
            "nmap", "nc", "wget", "grep", "awk", "sed",
            "find", "strings", "hexdump", "diff",
        ],
        caps: &["net"],
    },
    Kit {
        name: "wireless",
        description: "Wireless network attacks",
        tools: &[
            "ifconfig", "ip", "arp", "ping", "nc",
            "hexdump", "strings", "grep",
        ],
        caps: &["net", "raw"],
    },
    Kit {
        name: "webapp",
        description: "Web application analysis",
        tools: &[
            "wget", "nc", "grep", "sed", "awk", "tr", "cut",
            "sort", "uniq", "wc", "head", "tail", "find",
            "strings", "md5sum", "sha256sum",
        ],
        caps: &["net"],
    },
    Kit {
        name: "database",
        description: "Database assessment",
        tools: &[
            "nc", "grep", "awk", "sed", "sort", "uniq",
            "strings", "hexdump", "wget",
        ],
        caps: &["net"],
    },
    Kit {
        name: "passwords",
        description: "Password attacks & cracking",
        tools: &[
            "grep", "awk", "sed", "sort", "uniq", "tr", "cut",
            "wc", "md5sum", "sha1sum", "sha256sum", "sha512sum",
            "strings", "hexdump",
        ],
        caps: &["fs"],
    },
    Kit {
        name: "exploit",
        description: "Exploitation tools",
        tools: &[
            "nmap", "nc", "wget", "grep", "sed", "awk",
            "strings", "hexdump", "find", "env", "id",
            "uname", "hostname", "whoami",
        ],
        caps: &["net", "raw"],
    },
    Kit {
        name: "sniff",
        description: "Sniffing & spoofing",
        tools: &[
            "ifconfig", "ip", "arp", "nc", "hexdump",
            "strings", "grep", "netstat",
        ],
        caps: &["net", "raw"],
    },
    Kit {
        name: "postexploit",
        description: "Post-exploitation",
        tools: &[
            "id", "whoami", "uname", "hostname", "env",
            "ps", "ls", "cat", "find", "grep", "awk", "sed",
            "wget", "nc", "strings", "hexdump",
            "passwd", "su", "chmod", "chown",
        ],
        caps: &["net", "fs"],
    },
    Kit {
        name: "forensics",
        description: "Digital forensics",
        tools: &[
            "md5sum", "sha1sum", "sha256sum", "sha512sum",
            "strings", "hexdump", "find", "grep", "awk",
            "ls", "cat", "head", "tail", "wc", "sort", "diff",
        ],
        caps: &["fs"],
    },
    Kit {
        name: "reporting",
        description: "Reporting & documentation",
        tools: &[
            "cat", "echo", "grep", "awk", "sed", "sort",
            "wc", "head", "tail", "date", "hostname",
        ],
        caps: &["fs"],
    },
];

/// Find a kit by name.
pub fn find_kit(name: &str) -> Option<&'static Kit> {
    for kit in KITS {
        if kit.name == name { return Some(kit); }
    }
    None
}

/// Apply a kit to a BatCave: install all tools + grant all caps.
pub fn apply_kit(cave_name: &str, kit_name: &str) -> Result<(), &'static str> {
    let kit = find_kit(kit_name).ok_or("unknown kit")?;

    uart::puts("[kit] Applying '");
    uart::puts(kit.name);
    uart::puts("' kit to cave '");
    uart::puts(cave_name);
    uart::puts("'\n");

    // Install all tools
    for &tool in kit.tools {
        cave::install_tool(cave_name, tool).ok(); // ignore "already installed"
    }

    // Grant capabilities
    for &cap in kit.caps {
        cave::grant_cap(cave_name, cap).ok(); // ignore "already granted"
    }

    uart::puts("[kit] Installed ");
    crate::kernel::mm::print_num(kit.tools.len());
    uart::puts(" tools, granted ");
    crate::kernel::mm::print_num(kit.caps.len());
    uart::puts(" caps\n");

    Ok(())
}

/// List all available kits.
pub fn list_kits<F: FnMut(&str, &str, usize)>(mut f: F) {
    for kit in KITS {
        f(kit.name, kit.description, kit.tools.len());
    }
}
