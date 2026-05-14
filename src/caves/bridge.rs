#![allow(dead_code)]
// Sphragis — Bridge: Universal Tool-to-Tool Data Bridge
//
// Parses output from one security tool into a standard intermediate format,
// then converts it into input for another tool. No manual copy-paste.
//
// Example flows:
//   nmap scan → Bridge → metasploit resource script
//   nmap scan → Bridge → sqlmap targets
//   nikto scan → Bridge → burp sitemap
//   gobuster → Bridge → sqlmap URLs
//   enum4linux → Bridge → hydra target list
//
// Supported data types:
//   - Hosts (IP addresses + hostnames)
//   - Ports (IP:port + service + version)
//   - URLs (full HTTP/HTTPS URLs)
//   - Credentials (username:password pairs)
//   - Vulnerabilities (CVE + description + severity)


// ─── Standard Intermediate Format ───

const MAX_ENTRIES: usize = 128;
const MAX_FIELD: usize = 64;

#[derive(Clone, Copy, PartialEq)]
pub enum DataType {
    Empty,
    Host,       // IP address or hostname
    Port,       // IP:port with service info
    Url,        // Full URL
    Credential, // username:password
    Vuln,       // CVE/vulnerability
}

#[derive(Clone, Copy)]
pub struct PipeEntry {
    pub dtype: DataType,
    pub field1: [u8; MAX_FIELD],  // host/IP
    pub f1_len: usize,
    pub field2: [u8; MAX_FIELD],  // port/path/username
    pub f2_len: usize,
    pub field3: [u8; MAX_FIELD],  // service/password/CVE
    pub f3_len: usize,
    pub severity: u8,             // 0=info, 1=low, 2=med, 3=high, 4=critical
}

impl PipeEntry {
    const fn empty() -> Self {
        PipeEntry {
            dtype: DataType::Empty,
            field1: [0; MAX_FIELD], f1_len: 0,
            field2: [0; MAX_FIELD], f2_len: 0,
            field3: [0; MAX_FIELD], f3_len: 0,
            severity: 0,
        }
    }

    pub fn host(ip: &[u8]) -> Self {
        let mut e = Self::empty();
        e.dtype = DataType::Host;
        e.f1_len = ip.len().min(MAX_FIELD);
        e.field1[..e.f1_len].copy_from_slice(&ip[..e.f1_len]);
        e
    }

    pub fn port(ip: &[u8], port: &[u8], service: &[u8]) -> Self {
        let mut e = Self::empty();
        e.dtype = DataType::Port;
        e.f1_len = ip.len().min(MAX_FIELD);
        e.field1[..e.f1_len].copy_from_slice(&ip[..e.f1_len]);
        e.f2_len = port.len().min(MAX_FIELD);
        e.field2[..e.f2_len].copy_from_slice(&port[..e.f2_len]);
        e.f3_len = service.len().min(MAX_FIELD);
        e.field3[..e.f3_len].copy_from_slice(&service[..e.f3_len]);
        e
    }

    pub fn url(full_url: &[u8]) -> Self {
        let mut e = Self::empty();
        e.dtype = DataType::Url;
        e.f1_len = full_url.len().min(MAX_FIELD);
        e.field1[..e.f1_len].copy_from_slice(&full_url[..e.f1_len]);
        e
    }

    pub fn credential(user: &[u8], pass: &[u8]) -> Self {
        let mut e = Self::empty();
        e.dtype = DataType::Credential;
        e.f1_len = user.len().min(MAX_FIELD);
        e.field1[..e.f1_len].copy_from_slice(&user[..e.f1_len]);
        e.f2_len = pass.len().min(MAX_FIELD);
        e.field2[..e.f2_len].copy_from_slice(&pass[..e.f2_len]);
        e
    }

    pub fn vuln(cve: &[u8], desc: &[u8], severity: u8) -> Self {
        let mut e = Self::empty();
        e.dtype = DataType::Vuln;
        e.f1_len = cve.len().min(MAX_FIELD);
        e.field1[..e.f1_len].copy_from_slice(&cve[..e.f1_len]);
        e.f2_len = desc.len().min(MAX_FIELD);
        e.field2[..e.f2_len].copy_from_slice(&desc[..e.f2_len]);
        e.severity = severity;
        e
    }

    pub fn f1_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.field1[..self.f1_len]) }
    }
    pub fn f2_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.field2[..self.f2_len]) }
    }
    pub fn f3_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.field3[..self.f3_len]) }
    }
}

// ─── Pipe Buffer (shared between tools) ───

static mut PIPE_BUF: [PipeEntry; MAX_ENTRIES] = [PipeEntry::empty(); MAX_ENTRIES];
static mut PIPE_COUNT: usize = 0;
static mut PIPE_NAME: [u8; 32] = [0; 32];
static mut PIPE_NAME_LEN: usize = 0;

/// Clear the pipe buffer.
pub fn clear() {
    unsafe {
        PIPE_COUNT = 0;
        for i in 0..MAX_ENTRIES { PIPE_BUF[i] = PipeEntry::empty(); }
    }
}

/// V8-ROOT-2 (V10 regression fix): clear the inter-tool batpipe (nmap/etc.
/// output) AND its name on cave switch. clear() alone leaves PIPE_NAME
/// intact, which discloses the previous cave's last running tool.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    clear();
    unsafe {
        PIPE_NAME = [0; 32];
        PIPE_NAME_LEN = 0;
    }
}

/// Set a name for this pipe (e.g., "nmap-scan-1").
pub fn set_name(name: &str) {
    unsafe {
        PIPE_NAME_LEN = name.len().min(32);
        PIPE_NAME[..PIPE_NAME_LEN].copy_from_slice(&name.as_bytes()[..PIPE_NAME_LEN]);
    }
}

/// Add an entry to the pipe.
pub fn push(entry: PipeEntry) -> Result<(), &'static str> {
    unsafe {
        if PIPE_COUNT >= MAX_ENTRIES { return Err("pipe full"); }
        PIPE_BUF[PIPE_COUNT] = entry;
        PIPE_COUNT += 1;
    }
    Ok(())
}

/// Get pipe entry count.
pub fn count() -> usize { unsafe { PIPE_COUNT } }

/// Iterate over all entries.
pub fn each<F: FnMut(&PipeEntry)>(mut f: F) {
    unsafe {
        for i in 0..PIPE_COUNT {
            f(&PIPE_BUF[i]);
        }
    }
}

// ─── Parsers: Tool Output → Pipe Entries ───

/// Parse nmap-style output (simplified).
/// Looks for lines like: "22/tcp open ssh OpenSSH 8.9"
pub fn parse_nmap(output: &[u8], target_ip: &[u8]) {
    clear();
    set_name("nmap");
    push(PipeEntry::host(target_ip)).ok();

    let mut i = 0;
    while i < output.len() {
        // Find lines containing "/tcp" or "/udp"
        let line_start = i;
        while i < output.len() && output[i] != b'\n' { i += 1; }
        let line = &output[line_start..i];
        i += 1;

        // Look for port/proto pattern: "22/tcp"
        if let Some(slash) = line.iter().position(|&b| b == b'/') {
            if slash > 0 && slash + 3 < line.len() {
                let port = &line[..slash];
                let proto = &line[slash + 1..];

                // Check for "tcp" or "udp"
                if proto.starts_with(b"tcp") || proto.starts_with(b"udp") {
                    // Extract service name (after "open ")
                    let mut service = b"unknown" as &[u8];
                    if let Some(open_pos) = find_bytes(line, b"open") {
                        let svc_start = open_pos + 5; // skip "open "
                        if svc_start < line.len() {
                            let mut svc_end = svc_start;
                            while svc_end < line.len() && line[svc_end] != b' ' { svc_end += 1; }
                            service = &line[svc_start..svc_end];
                        }
                    }
                    push(PipeEntry::port(target_ip, port, service)).ok();
                }
            }
        }
    }
}

/// Parse URL list (one URL per line).
pub fn parse_urls(output: &[u8]) {
    clear();
    set_name("urls");
    let mut i = 0;
    while i < output.len() {
        let line_start = i;
        while i < output.len() && output[i] != b'\n' { i += 1; }
        let line = &output[line_start..i];
        i += 1;
        if line.starts_with(b"http://") || line.starts_with(b"https://") {
            push(PipeEntry::url(line)).ok();
        }
    }
}

// ─── Exporters: Pipe Entries → Tool Input ───

/// Export as Metasploit resource script.
/// Generates: use exploit/...; set RHOSTS ...; set RPORT ...;
pub fn export_metasploit(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    let header = b"# Bridge -> Metasploit Resource Script\n";
    if pos + header.len() < buf.len() {
        buf[pos..pos + header.len()].copy_from_slice(header);
        pos += header.len();
    }

    unsafe {
        for i in 0..PIPE_COUNT {
            let e = &PIPE_BUF[i];
            match e.dtype {
                DataType::Port => {
                    // set RHOSTS <ip>
                    let cmd = b"set RHOSTS ";
                    if pos + cmd.len() + e.f1_len + 1 < buf.len() {
                        buf[pos..pos + cmd.len()].copy_from_slice(cmd);
                        pos += cmd.len();
                        buf[pos..pos + e.f1_len].copy_from_slice(&e.field1[..e.f1_len]);
                        pos += e.f1_len;
                        buf[pos] = b'\n'; pos += 1;
                    }
                    // set RPORT <port>
                    let cmd = b"set RPORT ";
                    if pos + cmd.len() + e.f2_len + 1 < buf.len() {
                        buf[pos..pos + cmd.len()].copy_from_slice(cmd);
                        pos += cmd.len();
                        buf[pos..pos + e.f2_len].copy_from_slice(&e.field2[..e.f2_len]);
                        pos += e.f2_len;
                        buf[pos] = b'\n'; pos += 1;
                    }
                }
                _ => {}
            }
        }
    }
    pos
}

/// Export as sqlmap target list (-u URLs).
pub fn export_sqlmap(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    unsafe {
        for i in 0..PIPE_COUNT {
            let e = &PIPE_BUF[i];
            if e.dtype == DataType::Url {
                if pos + e.f1_len + 1 < buf.len() {
                    buf[pos..pos + e.f1_len].copy_from_slice(&e.field1[..e.f1_len]);
                    pos += e.f1_len;
                    buf[pos] = b'\n'; pos += 1;
                }
            }
        }
    }
    pos
}

/// Export as hydra target list (ip:port format).
pub fn export_hydra(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    unsafe {
        for i in 0..PIPE_COUNT {
            let e = &PIPE_BUF[i];
            if e.dtype == DataType::Port {
                if pos + e.f1_len + 1 + e.f2_len + 1 < buf.len() {
                    buf[pos..pos + e.f1_len].copy_from_slice(&e.field1[..e.f1_len]);
                    pos += e.f1_len;
                    buf[pos] = b':'; pos += 1;
                    buf[pos..pos + e.f2_len].copy_from_slice(&e.field2[..e.f2_len]);
                    pos += e.f2_len;
                    buf[pos] = b'\n'; pos += 1;
                }
            }
        }
    }
    pos
}

/// Export as simple host list (one IP per line).
pub fn export_hosts(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    unsafe {
        for i in 0..PIPE_COUNT {
            let e = &PIPE_BUF[i];
            if e.dtype == DataType::Host {
                if pos + e.f1_len + 1 < buf.len() {
                    buf[pos..pos + e.f1_len].copy_from_slice(&e.field1[..e.f1_len]);
                    pos += e.f1_len;
                    buf[pos] = b'\n'; pos += 1;
                }
            }
        }
    }
    pos
}

// ─── Helpers ───

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() > haystack.len() { return None; }
    for i in 0..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
    }
    None
}
