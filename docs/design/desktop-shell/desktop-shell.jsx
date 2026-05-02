// Bat_OS desktop chrome — title bar + content slot + status bar.
// Plus the SH (shell) pane content with three states.

const shellColors = {
  bg: "#0A0A0A",
  panel: "#0E0E0E",
  hair: "#1A1A1A",
  hairHi: "#262626",
  ink: "#E5E7EB",
  mid: "#9CA3AF",
  dim: "#4B5563",
  faint: "#374151",
  cyan: "#22D3EE",
  cyanDim: "#0E7490",
  green: "#22C55E",
  greenDim: "#14532D",
  amber: "#F59E0B",
  amberDim: "#78350F",
  red: "#EF4444",
  redDim: "#7F1D1D",
};

const shellMono = `"JetBrains Mono", "IBM Plex Mono", "SF Mono", Menlo, monospace`;

// — TITLE BAR ————————————————————————————————————————————————

const TABS = [
  { code: "SH", name: "shell"     },
  { code: "DS", name: "dashboard" },
  { code: "FS", name: "files"     },
  { code: "NM", name: "netmon"    },
  { code: "ED", name: "editor"    },
  { code: "SK", name: "security"  },
  { code: "CM", name: "comms"     },
  { code: "WB", name: "browser"   },
  { code: "BC", name: "batcave"   },
];

const TitleBar = ({ active = "SH", caveName = "kernel", caveStatus = "ok" }) => {
  const dotColor =
    caveStatus === "ok"   ? shellColors.green :
    caveStatus === "warn" ? shellColors.amber :
                            shellColors.red;
  const dotRing =
    caveStatus === "ok"   ? shellColors.greenDim :
    caveStatus === "warn" ? shellColors.amberDim :
                            shellColors.redDim;

  return (
    <div style={{
      position: "relative",
      height: 24, width: "100%",
      background: shellColors.bg,
      borderBottom: `1px solid ${shellColors.hair}`,
      display: "flex", alignItems: "stretch",
      fontFamily: shellMono,
    }}>
      {/* — Left: brand — */}
      <div style={{
        display: "flex", alignItems: "center", gap: 8,
        padding: "0 14px",
        borderRight: `1px solid ${shellColors.hair}`,
        minWidth: 132,
      }}>
        <BatMini size={18} color={shellColors.cyan} />
        <div style={{
          fontSize: 12, fontWeight: 700, letterSpacing: 2,
          color: shellColors.ink,
        }}>
          BAT<span style={{ color: shellColors.cyan }}>_</span>OS
        </div>
      </div>

      {/* — Center: tabs — */}
      <div style={{
        flex: 1,
        display: "flex", justifyContent: "center", alignItems: "stretch",
      }}>
        {TABS.map((t, i) => {
          const isActive = t.code === active;
          return (
            <div key={t.code} style={{
              position: "relative",
              width: 64,
              display: "flex", flexDirection: "column",
              justifyContent: "center", alignItems: "center",
              borderRight: i === TABS.length - 1 ? "none" : `1px solid ${shellColors.hair}`,
              cursor: "default",
            }}>
              <div style={{
                fontSize: 8, letterSpacing: 1,
                color: isActive ? shellColors.cyan : shellColors.faint,
                lineHeight: 1, marginTop: 2,
              }}>
                ⌃{i + 1}
              </div>
              <div style={{
                fontSize: 11, fontWeight: isActive ? 700 : 500,
                letterSpacing: 1.5,
                color: isActive ? shellColors.ink : shellColors.dim,
                lineHeight: 1, marginTop: 3,
              }}>
                {t.code}
              </div>
              {isActive && (
                <div style={{
                  position: "absolute", left: 6, right: 6, bottom: 0, height: 2,
                  background: shellColors.cyan,
                }} />
              )}
            </div>
          );
        })}
      </div>

      {/* — Right: cave indicator — */}
      <div style={{
        display: "flex", alignItems: "center", gap: 8,
        padding: "0 14px",
        borderLeft: `1px solid ${shellColors.hair}`,
        minWidth: 168, justifyContent: "flex-end",
      }}>
        <span style={{
          fontSize: 10, letterSpacing: 1.5, color: shellColors.dim,
          textTransform: "uppercase",
        }}>cave</span>
        <span style={{ fontSize: 11, color: shellColors.ink, letterSpacing: 1 }}>
          {caveName}
        </span>
        <span style={{
          width: 6, height: 6, background: dotColor,
          boxShadow: `0 0 0 1px ${dotRing}`,
          display: "inline-block",
        }} />
      </div>
    </div>
  );
};

// — STATUS BAR ———————————————————————————————————————————————

const StatusSeg = ({ label, value, valueColor, dot, dotColor, dotRing }) => (
  <div style={{
    display: "flex", alignItems: "center", gap: 8,
    padding: "0 12px",
    borderRight: `1px solid ${shellColors.hairHi}`,
    height: "100%",
  }}>
    {dot && (
      <span style={{
        width: 6, height: 6, background: dotColor,
        boxShadow: `0 0 0 1px ${dotRing}`,
        display: "inline-block",
      }} />
    )}
    <span style={{
      fontSize: 10, letterSpacing: 1.5, color: shellColors.mid,
      textTransform: "uppercase",
    }}>{label}</span>
    {value !== undefined && (
      <span style={{
        fontSize: 11, letterSpacing: 1, color: valueColor || shellColors.ink,
      }}>{value}</span>
    )}
  </div>
);

const StatusBar = ({
  net = "10.0.2.15",
  tlsMode = "LOCKDOWN",
  js = "OFF",
  audit = "247 / 1024",
  uptime = "0d 00:14:32",
  blink = true,
}) => {
  const tlsColor =
    tlsMode === "LOCKDOWN" ? shellColors.cyan :
    tlsMode === "RESEARCH" ? shellColors.amber :
                             shellColors.red;
  return (
    <div style={{
      position: "relative",
      height: 28, width: "100%",
      background: shellColors.bg,
      borderTop: `1px solid ${shellColors.hair}`,
      display: "flex", alignItems: "stretch",
      fontFamily: shellMono,
    }}>
      <StatusSeg
        label="ENCRYPTED"
        dot dotColor={shellColors.green} dotRing={shellColors.greenDim}
      />
      <StatusSeg label="NET" value={net} valueColor={net === "OFFLINE" ? shellColors.red : shellColors.ink} />
      <StatusSeg label="TLS" value={tlsMode} valueColor={tlsColor} />
      <StatusSeg label="JS"  value={js}      valueColor={js === "ON" ? shellColors.amber : shellColors.ink} />
      <StatusSeg label="AUDIT" value={audit} />

      <div style={{ flex: 1 }} />

      <div style={{
        display: "flex", alignItems: "center", gap: 8,
        padding: "0 12px",
        borderLeft: `1px solid ${shellColors.hairHi}`,
      }}>
        <span style={{
          fontSize: 10, letterSpacing: 1.5, color: shellColors.mid,
          textTransform: "uppercase",
        }}>uptime</span>
        <span style={{ fontSize: 11, letterSpacing: 1, color: shellColors.ink }}>
          {uptime.replace(/:/g, blink ? ":" : " ")}
        </span>
      </div>
    </div>
  );
};

// — SHELL PANE ———————————————————————————————————————————————

// One scrollback line. Categories drive color.
// kinds: "echo" | "out" | "audit" | "warn" | "err" | "banner" | "blank"
const Line = ({ kind = "out", children, indent = 0 }) => {
  const colorMap = {
    echo:   shellColors.dim,    // cmd echo prefix "bat_os >"
    out:    shellColors.ink,    // normal output
    audit:  shellColors.cyan,   // [243]
    warn:   shellColors.amber,
    err:    shellColors.red,
    banner: shellColors.mid,
    blank:  "transparent",
  };
  return (
    <div style={{
      fontFamily: shellMono, fontSize: 13, lineHeight: "16px",
      color: colorMap[kind], paddingLeft: indent,
      whiteSpace: "pre",
    }}>
      {children || "\u00A0"}
    </div>
  );
};

const Prompt = ({ typed = "", cursor = true, blink = true }) => (
  <div style={{
    fontFamily: shellMono, fontSize: 13, lineHeight: "16px",
    display: "flex", alignItems: "center",
  }}>
    <span style={{ color: shellColors.ink }}>bat_os</span>
    <span style={{ color: shellColors.cyan, padding: "0 6px" }}>&gt;</span>
    <span style={{ color: shellColors.ink, whiteSpace: "pre" }}>{typed}</span>
    {cursor && (
      <span style={{
        width: 8, height: 14, marginLeft: typed ? 1 : 0,
        background: shellColors.cyan,
        animation: blink ? "shellCursor 1s steps(2) infinite" : "none",
        display: "inline-block",
      }} />
    )}
  </div>
);

// Echo "bat_os > <cmd>" line, used inside scrollback
const Echo = ({ cmd }) => (
  <div style={{
    fontFamily: shellMono, fontSize: 13, lineHeight: "16px",
    whiteSpace: "pre",
  }}>
    <span style={{ color: shellColors.dim }}>bat_os</span>
    <span style={{ color: shellColors.cyanDim, padding: "0 6px" }}>&gt;</span>
    <span style={{ color: shellColors.ink }}>{cmd}</span>
  </div>
);

const ShellBanner = () => (
  <div style={{ fontFamily: shellMono, marginBottom: 8 }}>
    <div style={{ display: "flex", alignItems: "center", gap: 14, marginBottom: 6 }}>
      <BatGlyph size={36} stroke={shellColors.cyan} node={shellColors.cyan} dim={shellColors.cyanDim} />
      <div>
        <div style={{
          fontSize: 14, fontWeight: 700, letterSpacing: 4, color: shellColors.ink,
        }}>
          BAT<span style={{ color: shellColors.cyan }}>_</span>OS
          <span style={{ color: shellColors.dim, fontWeight: 400, marginLeft: 12, letterSpacing: 1 }}>
            v0.5.0-DEV
          </span>
          <span style={{ color: shellColors.mid, fontWeight: 400, marginLeft: 12, letterSpacing: 1 }}>
            Microkernel Shell
          </span>
        </div>
        <div style={{ fontSize: 11, color: shellColors.dim, letterSpacing: 1, marginTop: 4 }}>
          tab to switch apps
          <span style={{ color: shellColors.faint }}> · </span>
          ⌃1:SH ⌃2:DS ⌃3:FS ⌃4:NM ⌃5:ED ⌃6:SK ⌃7:CM ⌃8:WB ⌃9:BC
        </div>
        <div style={{ fontSize: 11, color: shellColors.dim, letterSpacing: 1, marginTop: 2 }}>
          type <span style={{ color: shellColors.cyan }}>help</span> for commands
          <span style={{ color: shellColors.faint }}> · </span>
          <span style={{ color: shellColors.cyan }}>tls-mode</span> · <span style={{ color: shellColors.cyan }}>render &lt;url&gt;</span> · <span style={{ color: shellColors.cyan }}>audit &lt;n&gt;</span> · <span style={{ color: shellColors.cyan }}>origin-allow</span>
        </div>
      </div>
    </div>
  </div>
);

// — Three pane states —

const ShellPaneEmpty = () => (
  <div style={{
    height: "100%", padding: "8px 16px",
    display: "flex", flexDirection: "column",
    background: shellColors.bg,
    fontFamily: shellMono,
  }}>
    <ShellBanner />
    <div style={{ flex: 1 }} />
    <Prompt typed="" cursor />
  </div>
);

const ShellPaneActivity = () => (
  <div style={{
    height: "100%", padding: "8px 16px",
    display: "flex", flexDirection: "column",
    background: shellColors.bg,
    fontFamily: shellMono,
    overflow: "hidden",
  }}>
    {/* hard top edge — older scrollback clips, no fade */}
    <div style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column", justifyContent: "flex-end" }}>
      <Echo cmd="tls-mode" />
      <Line>{"  tls-mode: "}<span style={{ color: shellColors.cyan }}>LOCKDOWN</span>{"  (1 pin · 0 mismatches)"}</Line>
      <Line kind="blank" />

      <Echo cmd="audit 5" />
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[243]</span>{" "}
        <span style={{ color: shellColors.mid }}>script:</span>{" exec js 1024B"}
      </Line>
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[242]</span>{" "}
        <span style={{ color: shellColors.mid }}>fetch :</span>{" GET http://10.0.2.2:8765/  "}
        <span style={{ color: shellColors.green }}>OK</span>{"  8421B"}
      </Line>
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[241]</span>{" "}
        <span style={{ color: shellColors.mid }}>nav   :</span>{" main origin -> http://10.0.2.2:8765"}
      </Line>
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[240]</span>{" "}
        <span style={{ color: shellColors.mid }}>mode  :</span>{" js-mode -> "}
        <span style={{ color: shellColors.amber }}>on</span>
      </Line>
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[239]</span>{" "}
        <span style={{ color: shellColors.mid }}>mode  :</span>{" tls-mode -> "}
        <span style={{ color: shellColors.amber }}>research</span>
      </Line>
      <Line kind="blank" />

      <Echo cmd="render file:///bin/login_test.html" />
      <Line>{"  render: fetched 1842 bytes"}</Line>
      <Line>{"  render: parsed 47 nodes"}</Line>
      <Line>{"  render: laid out 89 boxes"}</Line>
      <Line>{"  render: paint complete  "}<span style={{ color: shellColors.green }}>OK</span></Line>
      <Line kind="blank" />

      <Echo cmd="origin-allow batcave.local" />
      <Line kind="warn">{"  warn: origin batcave.local not in pinset; added (session-scoped)"}</Line>
      <Line kind="blank" />
    </div>
    <Prompt typed="" cursor />
  </div>
);

const ShellPaneTyping = () => (
  <div style={{
    height: "100%", padding: "8px 16px",
    display: "flex", flexDirection: "column",
    background: shellColors.bg,
    fontFamily: shellMono,
  }}>
    <div style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column", justifyContent: "flex-end" }}>
      <Echo cmd="audit 3" />
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[247]</span>{" "}
        <span style={{ color: shellColors.mid }}>origin:</span>{" allow batcave.local (session)"}
      </Line>
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[246]</span>{" "}
        <span style={{ color: shellColors.mid }}>cert  :</span>{" pin verify "}
        <span style={{ color: shellColors.green }}>OK</span>{"  cdn.example.com"}
      </Line>
      <Line>
        {"  "}<span style={{ color: shellColors.cyan }}>[245]</span>{" "}
        <span style={{ color: shellColors.mid }}>fetch :</span>{" HEAD https://example.com/  "}
        <span style={{ color: shellColors.red }}>FAIL</span>
      </Line>
      <Line kind="blank" />
      <Echo cmd="tls-mode lockdown" />
      <Line>{"  tls-mode: "}<span style={{ color: shellColors.cyan }}>LOCKDOWN</span>{"  pinset rebuilt (3 entries)"}</Line>
      <Line kind="blank" />
    </div>
    <Prompt typed="origin-allow example.com cdn.example." cursor />
  </div>
);

// — DESKTOP CHROME WRAPPER —

const Desktop = ({
  active = "SH",
  caveName = "kernel",
  caveStatus = "ok",
  net = "10.0.2.15",
  tlsMode = "LOCKDOWN",
  js = "OFF",
  audit = "247 / 1024",
  uptime = "0d 00:14:32",
  children,
  width = 1280, height = 800,
}) => (
  <div style={{
    width, height, background: shellColors.bg,
    display: "flex", flexDirection: "column",
    overflow: "hidden", position: "relative",
  }}>
    <TitleBar active={active} caveName={caveName} caveStatus={caveStatus} />
    {/* content area */}
    <div style={{
      flex: 1, overflow: "hidden",
      borderTop: `1px solid ${shellColors.hair}`,
      borderBottom: `1px solid ${shellColors.hair}`,
    }}>
      {children}
    </div>
    <StatusBar net={net} tlsMode={tlsMode} js={js} audit={audit} uptime={uptime} />
  </div>
);

window.Desktop = Desktop;
window.TitleBar = TitleBar;
window.StatusBar = StatusBar;
window.ShellPaneEmpty = ShellPaneEmpty;
window.ShellPaneActivity = ShellPaneActivity;
window.ShellPaneTyping = ShellPaneTyping;
