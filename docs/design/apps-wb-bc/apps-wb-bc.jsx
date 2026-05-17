// WB · BC — Sphragis Wave 4
// Reuses appColors / appMono / Panel / KV / Dot / CaveRow / CaveEmptyRow / AuditStrip
// from app-panels.jsx, plus Strip / StripSeg / ConnPill from apps-fs-ed-cm.jsx.

// — Tiny helpers ————————————————————————————————————————————

// host -> 12x12 swatch color, deterministic hash → pick from palette
const swatchColor = (host) => {
  const palette = [
    appColors.cyan, appColors.green, appColors.amber, appColors.red,
    appColors.cyanDim, appColors.greenDim, appColors.amberDim, appColors.redDim,
  ];
  let h = 0;
  for (let i = 0; i < host.length; i++) h = (h * 31 + host.charCodeAt(i)) | 0;
  return palette[Math.abs(h) % palette.length];
};

// 14x14 lock glyph using "[]" framing — green/amber/red/dim per state
const Lock = ({ kind = "https" }) => {
  const map = {
    https:    { c: appColors.green, ring: appColors.greenDim, glyph: "[#]" },
    research: { c: appColors.amber, ring: appColors.amberDim, glyph: "[#]" },
    http:     { c: appColors.red,   ring: appColors.redDim,   glyph: "[/]" },
    file:     { c: appColors.dim,   ring: appColors.faint,    glyph: " ? " },
  };
  const v = map[kind] || map.file;
  return (
    <span style={{
      width: 22, textAlign: "center",
      color: v.c, fontFamily: appMono, fontSize: 11,
      letterSpacing: 0, lineHeight: 1,
      borderRight: `1px solid ${appColors.hair}`,
      paddingRight: 6, marginRight: 8,
    }}>{v.glyph}</span>
  );
};

const NavBtn = ({ glyph, enabled = true, loading = false }) => (
  <div style={{
    width: 28, height: 28,
    border: `1px solid ${appColors.hairHi}`,
    background: appColors.panel,
    display: "flex", alignItems: "center", justifyContent: "center",
    color: loading ? appColors.amber : (enabled ? appColors.cyan : appColors.faint),
    fontFamily: appMono, fontSize: 13, fontWeight: 700,
    flexShrink: 0,
  }}>
    {glyph}
  </div>
);

// — WB · BROWSER ————————————————————————————————————————————

const URLBar = ({ value, focused, cursorAt, lockKind }) => (
  <div style={{
    flex: 1, height: 28,
    border: `1px solid ${focused ? appColors.cyan : appColors.hairHi}`,
    background: appColors.panel,
    display: "flex", alignItems: "center",
    padding: "0 10px",
    fontFamily: appMono, fontSize: 12,
    minWidth: 0,
  }}>
    <Lock kind={lockKind} />
    <span style={{
      color: appColors.ink, whiteSpace: "pre",
      overflow: "hidden", textOverflow: "ellipsis",
      flexShrink: 1, minWidth: 0,
    }}>
      {value.slice(0, cursorAt)}
    </span>
    {focused && (
      <span style={{
        display: "inline-block", width: 8, height: 2,
        background: appColors.cyan,
        marginLeft: 1, marginBottom: -8,
        animation: "shellCursor 1s steps(2) infinite",
      }} />
    )}
    <span style={{ color: appColors.ink, whiteSpace: "pre" }}>
      {value.slice(cursorAt)}
    </span>
  </div>
);

const SOPPill = ({ tone, host }) => {
  const c = tone === "strict" ? appColors.cyan : tone === "loose" ? appColors.amber : appColors.red;
  const dim = tone === "strict" ? appColors.cyanDim : tone === "loose" ? appColors.amberDim : appColors.redDim;
  return (
    <div style={{
      display: "inline-flex", alignItems: "center", gap: 6,
      padding: "0 8px", height: 22,
      border: `1px solid ${appColors.hairHi}`,
      background: appColors.panel,
      fontSize: 10, letterSpacing: 1.5, color: appColors.mid,
      textTransform: "uppercase",
    }}>
      <span style={{ color: c, fontWeight: 700 }}>SOP</span>
      <span style={{ color: appColors.ink, textTransform: "none", letterSpacing: 0 }}>
        {host || "—"}
      </span>
    </div>
  );
};

const JSPill = ({ on }) => (
  <div style={{
    display: "inline-flex", alignItems: "center", gap: 6,
    padding: "0 8px", height: 22,
    border: `1px solid ${appColors.hairHi}`,
    background: appColors.panel,
    fontSize: 10, letterSpacing: 1.5,
    color: on ? appColors.amber : appColors.ink,
    textTransform: "uppercase",
  }}>
    JS {on ? "ON" : "OFF"}
  </div>
);

const StarBtn = ({ on }) => (
  <div style={{
    width: 28, height: 28,
    border: `1px solid ${appColors.hairHi}`,
    background: appColors.panel,
    display: "flex", alignItems: "center", justifyContent: "center",
    color: on ? appColors.cyan : appColors.faint,
    fontFamily: appMono, fontSize: 14, fontWeight: 700,
    flexShrink: 0,
  }}>*</div>
);

const Bookmark = ({ host, sub }) => (
  <div style={{
    display: "inline-flex", alignItems: "center", gap: 8,
    height: 24, padding: "0 12px",
    fontFamily: appMono, fontSize: 11, color: appColors.ink,
    flexShrink: 0,
  }}>
    <span style={{
      width: 12, height: 12, background: swatchColor(host),
      boxShadow: `0 0 0 1px ${appColors.hair}`,
      display: "inline-block", flexShrink: 0,
    }} />
    <span style={{ letterSpacing: 0 }}>{host}</span>
    {sub && <span style={{ color: appColors.dim, fontSize: 10 }}>{sub}</span>}
  </div>
);

// Faux page render
const FauxPage = () => (
  <div style={{
    fontFamily: appMono, fontSize: 13, lineHeight: "20px",
    padding: "24px 32px", color: appColors.ink,
    height: "100%", overflow: "hidden",
  }}>
    <div style={{
      fontSize: 22, fontWeight: 700, letterSpacing: 4,
      color: appColors.ink, marginBottom: 4,
    }}>
      BAT<span style={{ color: appColors.cyan }}>_</span>OS
      <span style={{ color: appColors.mid, fontWeight: 400, marginLeft: 12, letterSpacing: 1, fontSize: 14 }}>
        Operator Workstation
      </span>
    </div>
    <div style={{
      width: 320, height: 1, background: appColors.hairHi, margin: "8px 0 16px",
    }} />
    <p style={{ margin: "0 0 14px", maxWidth: 720 }}>
      Sphragis is a bare-metal AArch64 microkernel for security operators.
      Zero external dependencies. Audit-everything. Encrypted at rest with
      <span style={{ color: appColors.amber }}> AES-256-CTR </span>
      and SHA-256 integrity.
    </p>
    <div style={{
      fontSize: 14, fontWeight: 700, color: appColors.ink,
      marginTop: 18, marginBottom: 4, letterSpacing: 1,
    }}>What's inside</div>
    <ul style={{ margin: 0, padding: "0 0 0 20px", color: appColors.ink, listStyle: "square" }}>
      <li>Cave isolation — sealed containers with cap-set enforcement</li>
      <li>SealFS encrypted vault with merkle integrity</li>
      <li>TLS 1.3 stack with cert pinning · <span style={{ color: appColors.cyan }}>read more</span></li>
      <li>Hand-written shell with audit trail · <span style={{ color: appColors.cyan }}>read more</span></li>
    </ul>
    <div style={{
      fontSize: 14, fontWeight: 700, color: appColors.ink,
      marginTop: 18, marginBottom: 4, letterSpacing: 1,
    }}>Try it</div>
    <p style={{ margin: "0 0 8px", color: appColors.ink }}>
      In the shell: <span style={{ color: appColors.amber }}>render file:///bin/index.html</span>,
      then <span style={{ color: appColors.amber }}>tls-mode lockdown</span> to verify pinning.
    </p>
    <p style={{ margin: 0, color: appColors.dim, fontSize: 11 }}>
      example.com · served by sphragis/0.5.0-DEV · build 20260502.a3f1c
    </p>
  </div>
);

const WBPane = ({ state = "loaded" }) => {
  const isIdle    = state === "idle";
  const isLoading = state === "loading";

  let url, lockKind, sopTone, sopHost, statusLeft, progress, cookies;
  if (isIdle) {
    url = ""; lockKind = "file"; sopTone = "none"; sopHost = "";
    statusLeft = { text: "READY", color: appColors.dim };
    progress = 0; cookies = 0;
  } else if (isLoading) {
    url = "https://example.com/"; lockKind = "research";
    sopTone = "loose"; sopHost = "example.com";
    statusLeft = { text: "TLS HANDSHAKE…", color: appColors.amber };
    progress = 0.6; cookies = 0;
  } else {
    url = "https://example.com/"; lockKind = "https";
    sopTone = "strict"; sopHost = "example.com";
    statusLeft = { text: "RENDERED 8421B / 47 nodes / 89 boxes", color: appColors.ink };
    progress = 1; cookies = 3;
  }

  return (
    <div style={{
      width: "100%", height: "100%", background: appColors.bg,
      display: "flex", flexDirection: "column",
      fontFamily: appMono, overflow: "hidden",
    }}>
      {/* NAV STRIP */}
      <div style={{
        height: 40, flexShrink: 0,
        display: "flex", alignItems: "center", gap: 6,
        padding: "0 12px",
        borderBottom: `1px solid ${appColors.hair}`,
      }}>
        <NavBtn glyph="<" enabled={!isIdle} />
        <NavBtn glyph=">" enabled={false} />
        <NavBtn glyph="R" loading={isLoading} />
        <div style={{ width: 8 }} />
        <URLBar value={url} focused={isIdle} cursorAt={url.length} lockKind={lockKind} />
        <div style={{ width: 8 }} />
        <SOPPill tone={sopTone} host={sopHost} />
        <JSPill on={false} />
        <StarBtn on={state === "loaded"} />
      </div>

      {/* BOOKMARKS BAR */}
      <div style={{
        height: 24, flexShrink: 0,
        display: "flex", alignItems: "center",
        padding: "0 8px",
        borderBottom: `1px solid ${appColors.hair}`,
        background: appColors.bg,
      }}>
        <Bookmark host="home"  />
        <Bookmark host="shell" />
        <Bookmark host="docs"  />
        <Bookmark host="feed"  />
        <Bookmark host="ddg"   />
        <Bookmark host="hn"    />
        <div style={{ flex: 1 }} />
        <span style={{
          fontSize: 9, letterSpacing: 1.5, color: appColors.faint,
          textTransform: "uppercase", paddingRight: 8,
        }}>6 / 32 saved</span>
      </div>

      {/* PAGE AREA */}
      <div style={{ flex: 1, overflow: "hidden", position: "relative" }}>
        {state === "loaded" ? (
          <FauxPage />
        ) : (
          <div style={{
            width: "100%", height: "100%",
            display: "flex", alignItems: "center", justifyContent: "center",
            color: appColors.faint, fontSize: 11, letterSpacing: 2,
            textTransform: "uppercase",
          }}>
            {isIdle
              ? "[ enter a url to render ]"
              : "[ loading · framebuffer pending ]"}
          </div>
        )}
      </div>

      {/* STATUS STRIP */}
      <div style={{
        height: 24, flexShrink: 0,
        display: "flex", alignItems: "stretch",
        borderTop: `1px solid ${appColors.hair}`,
      }}>
        <div style={{
          width: 320,
          display: "flex", alignItems: "center", gap: 8,
          padding: "0 12px",
          borderRight: `1px solid ${appColors.hairHi}`,
          fontSize: 10, letterSpacing: 1.5,
          color: statusLeft.color, textTransform: "uppercase",
        }}>{statusLeft.text}</div>
        {/* progress */}
        <div style={{ flex: 1, position: "relative", display: "flex", alignItems: "center", padding: "0 12px" }}>
          {progress > 0 && progress < 1 && (
            <div style={{
              position: "relative", width: "100%", height: 1,
              background: appColors.cyanDim,
            }}>
              <div style={{
                position: "absolute", left: 0, top: 0, bottom: 0,
                width: `${progress * 100}%`,
                background: appColors.cyan,
              }} />
            </div>
          )}
        </div>
        <div style={{
          display: "flex", alignItems: "center", gap: 8,
          padding: "0 12px",
          borderLeft: `1px solid ${appColors.hairHi}`,
          fontSize: 10, letterSpacing: 1.5, color: appColors.dim,
          textTransform: "uppercase",
        }}>
          <span style={{
            width: 6, height: 6,
            background: cookies > 0 ? appColors.green : appColors.faint,
            boxShadow: `0 0 0 1px ${cookies > 0 ? appColors.greenDim : appColors.hair}`,
            display: "inline-block",
          }} />
          <span style={{ color: cookies > 0 ? appColors.ink : appColors.dim, fontVariantNumeric: "tabular-nums" }}>
            {cookies}
          </span>
          <span>cookies</span>
        </div>
      </div>
    </div>
  );
};

// — BC · CAVE ————————————————————————————————————————————

const CaveGlyph = ({ color = appColors.cyan }) => (
  <svg width="64" height="48" viewBox="0 0 64 48" shapeRendering="crispEdges">
    {/* outer container */}
    <rect x="6" y="8" width="52" height="32" fill="none" stroke={color} strokeWidth="1.5" />
    {/* inner dashed seal */}
    <rect x="12" y="14" width="40" height="20" fill="none" stroke={color} strokeWidth="1" strokeDasharray="3 2" opacity="0.6" />
    {/* corner notches */}
    <rect x="2"  y="4"  width="6" height="1" fill={color} />
    <rect x="2"  y="4"  width="1" height="6" fill={color} />
    <rect x="56" y="4"  width="6" height="1" fill={color} />
    <rect x="61" y="4"  width="1" height="6" fill={color} />
    <rect x="2"  y="43" width="6" height="1" fill={color} />
    <rect x="2"  y="38" width="1" height="6" fill={color} />
    <rect x="56" y="43" width="6" height="1" fill={color} />
    <rect x="61" y="38" width="1" height="6" fill={color} />
    {/* center node */}
    <rect x="31" y="23" width="2" height="2" fill={color} />
  </svg>
);

const BCCaveRow = ({ state, badge, name, type, caps, selected }) => {
  const cap = (n, on) => (
    <span key={n} style={{
      width: 38, height: 16,
      display: "inline-flex", alignItems: "center", justifyContent: "center",
      fontSize: 9, letterSpacing: 1,
      color: on ? appColors.cyan : appColors.faint,
      border: `1px solid ${on ? appColors.cyanDim : appColors.hair}`,
      background: appColors.panel,
      flexShrink: 0,
    }}>{n}</span>
  );
  const stateColors = {
    ok:   { c: appColors.green, dim: appColors.greenDim },
    plan: { c: appColors.dim,   dim: appColors.faint },
    warn: { c: appColors.amber, dim: appColors.amberDim },
    fail: { c: appColors.red,   dim: appColors.redDim },
  };
  const sc = stateColors[state] || stateColors.plan;
  const typeColor = type === "PERS" ? appColors.cyan : appColors.amber;
  const typeDim   = type === "PERS" ? appColors.cyanDim : appColors.amberDim;
  return (
    <div style={{ position: "relative", padding: "0 16px" }}>
      <div style={{
        display: "grid",
        gridTemplateColumns: "60px 1fr 60px 200px",
        height: 28, alignItems: "center", gap: 12,
        fontSize: 12, color: appColors.ink,
        border: selected ? `1px solid ${appColors.cyanDim}` : "1px solid transparent",
        borderBottom: `1px solid ${selected ? appColors.cyanDim : appColors.hair}`,
      }}>
        <span style={{
          width: 32, height: 16,
          display: "inline-flex", alignItems: "center", justifyContent: "center",
          fontSize: 9, letterSpacing: 1, color: sc.c,
          border: `1px solid ${sc.c}`, background: appColors.panel,
        }}>{badge}</span>
        <span style={{ color: appColors.ink }}>{name}</span>
        <span style={{
          width: 44, height: 16,
          display: "inline-flex", alignItems: "center", justifyContent: "center",
          fontSize: 9, letterSpacing: 1, color: typeColor,
          border: `1px solid ${typeDim}`, background: appColors.panel,
        }}>{type}</span>
        <span style={{ display: "inline-flex", gap: 4 }}>
          {["NET","RAW","DSP","FS"].map((n) => cap(n, caps.includes(n)))}
        </span>
      </div>
      {selected && (
        <div style={{
          position: "absolute", left: 16, right: 16, bottom: 0, height: 2,
          background: appColors.cyan,
        }} />
      )}
    </div>
  );
};

const BCTableHeader = () => (
  <div style={{
    display: "grid",
    gridTemplateColumns: "60px 1fr 60px 200px",
    gap: 12, padding: "0 16px",
    height: 22, alignItems: "center",
    fontSize: 9, letterSpacing: 1.5, color: appColors.faint,
    textTransform: "uppercase",
    borderBottom: `1px solid ${appColors.hair}`,
  }}>
    <span>STATE</span>
    <span>NAME</span>
    <span>TYPE</span>
    <span>CAPABILITIES</span>
  </div>
);

const ActionHint = ({ cmd, comment, danger }) => (
  <div style={{
    fontFamily: appMono, fontSize: 11, lineHeight: "18px",
    display: "flex", gap: 12, alignItems: "baseline",
  }}>
    <span style={{
      color: appColors.faint, letterSpacing: 1, textTransform: "uppercase",
    }}>[shell]</span>
    <span style={{ color: danger ? appColors.amber : appColors.cyan }}>
      {cmd}
    </span>
    {comment && (
      <span style={{ color: appColors.faint, marginLeft: "auto" }}>
        # {comment}
      </span>
    )}
  </div>
);

const QuickStartLine = ({ cmd, comment }) => (
  <div style={{
    fontFamily: appMono, fontSize: 11, lineHeight: "18px",
    display: "flex", gap: 12, alignItems: "baseline",
  }}>
    <span style={{ color: appColors.cyan }}>{cmd}</span>
    {comment && (
      <span style={{ color: appColors.faint, marginLeft: "auto" }}>
        # {comment}
      </span>
    )}
  </div>
);

const BCDetailEmpty = () => (
  <div style={{ padding: "24px 16px", height: "100%", display: "flex", flexDirection: "column" }}>
    <div style={{
      textAlign: "center", color: appColors.dim, fontSize: 12,
      letterSpacing: 1, marginBottom: 16,
    }}>(no cave selected)</div>
    <div style={{
      fontSize: 10, letterSpacing: 2, color: appColors.faint,
      textTransform: "uppercase", marginBottom: 8,
    }}>quick start</div>
    <QuickStartLine cmd="caves create pentest-lab --tools nmap,burpsuite" comment="docker-backed" />
    <QuickStartLine cmd="caves grant pentest-lab net" comment="grant capability" />
    <QuickStartLine cmd="caves grant pentest-lab raw display" comment="multiple at once" />
    <QuickStartLine cmd="caves enter pentest-lab" comment="attach to shell" />
    <QuickStartLine cmd="caves seal pentest-lab" comment="persistent → ephemeral" />
    <QuickStartLine cmd="caves destroy pentest-lab" comment="secure wipe" />
  </div>
);

const BCDetailCave = ({ cave }) => {
  const { name, state, type, fsKey, caps, tools, audit, created, wipe, auditLog } = cave;
  const stateMap = {
    ok:   { label: "RUNNING",  color: appColors.green },
    plan: { label: "STOPPED",  color: appColors.dim   },
    warn: { label: "WIPE",     color: appColors.amber },
  };
  const s = stateMap[state] || stateMap.plan;
  const glyphColor = state === "warn" ? appColors.amber : (state === "ok" ? appColors.cyan : appColors.dim);
  return (
    <div style={{ padding: "16px 16px 12px", height: "100%", display: "flex", flexDirection: "column", overflow: "hidden" }}>
      {/* glyph header */}
      <div style={{ display: "flex", alignItems: "center", gap: 16, marginBottom: 12 }}>
        <CaveGlyph color={glyphColor} />
        <div>
          <div style={{
            fontSize: 16, color: appColors.ink, letterSpacing: 1, fontWeight: 700,
          }}>
            {name}
          </div>
          <div style={{
            fontSize: 10, letterSpacing: 1.5, color: appColors.dim,
            textTransform: "uppercase", marginTop: 2,
          }}>
            <span style={{ color: s.color }}>● {s.label}</span>
            <span style={{ color: appColors.faint, margin: "0 8px" }}>·</span>
            <span style={{ color: type === "PERS" ? appColors.cyan : appColors.amber }}>
              {type === "PERS" ? "PERSISTENT" : "EPHEMERAL"}
            </span>
          </div>
        </div>
      </div>

      {/* KV rows */}
      <div style={{ marginBottom: 12 }}>
        <KV label="NAME"   value={name}   state="neutral" labelW={72} />
        <KV label="STATE"  value={s.label} state={state === "warn" ? "warn" : (state === "ok" ? "ok" : "plan")} dot={state === "warn" ? "warn" : (state === "ok" ? "ok" : "plan")} labelW={72} />
        <KV label="TYPE"   value={type === "PERS" ? "PERSISTENT" : "EPHEMERAL"} state={type === "PERS" ? "neutral" : "warn"} labelW={72} />
        <KV label="FS_KEY" value={wipe ? "wiped" : fsKey} state={wipe ? "fail" : "neutral"} labelW={72} />
        <KV label="CAPS"   value={caps.length ? caps.join(" ") : "—"} state={caps.length ? "neutral" : "plan"} labelW={72} />
        <KV label="TOOLS"  value={tools === 0 ? "0 (kernel cave)" : `${tools}`} state={tools === 0 ? "plan" : "neutral"} labelW={72} />
        <KV label="AUDIT"  value={`${audit} events`} state="neutral" labelW={72} />
        <KV label="CREATED" value={created} state="neutral" labelW={72} />
      </div>

      {/* action hints */}
      <div style={{
        fontSize: 10, letterSpacing: 2, color: appColors.faint,
        textTransform: "uppercase", marginBottom: 6,
      }}>actions</div>
      <ActionHint cmd={`caves enter ${name}`} comment="attach shell" />
      <ActionHint cmd={`caves seal ${name}`} comment="irreversible" danger />
      <ActionHint cmd={`caves destroy ${name}`} comment="secure wipe" danger />

      <div style={{ flex: 1 }} />

      {/* audit mini-strip */}
      <div style={{
        marginTop: 12, paddingTop: 8,
        borderTop: `1px solid ${appColors.hair}`,
      }}>
        <AuditStrip lines={auditLog} />
      </div>
    </div>
  );
};

const BCBottomStrip = ({ count, max, running, stopped, deleted }) => (
  <div style={{
    height: 28, flexShrink: 0,
    display: "flex", alignItems: "stretch",
    borderTop: `1px solid ${appColors.hair}`,
    fontFamily: appMono,
  }}>
    <div style={{
      display: "flex", alignItems: "center", gap: 8,
      padding: "0 16px",
      borderRight: `1px solid ${appColors.hairHi}`,
      fontSize: 10, letterSpacing: 1.5, color: appColors.mid,
      textTransform: "uppercase",
    }}>
      <span style={{ color: appColors.faint }}>CAVES</span>
      <span style={{ color: appColors.ink, fontVariantNumeric: "tabular-nums" }}>{count}</span>
      <span>·</span>
      <span style={{ color: appColors.faint }}>MAX</span>
      <span style={{ color: appColors.ink }}>{max}</span>
      <span>·</span>
      <span style={{ color: appColors.faint }}>RUNNING</span>
      <span style={{ color: appColors.ink }}>{running}</span>
    </div>
    <div style={{
      display: "flex", alignItems: "center", gap: 8,
      padding: "0 16px",
      borderRight: `1px solid ${appColors.hairHi}`,
    }}>
      {[
        { label: "RUN", value: running, color: appColors.green, dim: appColors.greenDim },
        { label: "STP", value: stopped, color: appColors.dim,   dim: appColors.faint },
        { label: "DEL", value: deleted, color: appColors.red,   dim: appColors.redDim },
      ].map((p) => (
        <span key={p.label} style={{
          display: "inline-flex", alignItems: "center", gap: 6,
          padding: "0 8px", height: 18,
          border: `1px solid ${p.dim}`,
          fontSize: 9, letterSpacing: 1.5, color: p.color,
        }}>
          {p.label}
          <span style={{ color: appColors.ink, fontVariantNumeric: "tabular-nums" }}>{p.value}</span>
        </span>
      ))}
    </div>
    <div style={{ flex: 1 }} />
    <div style={{
      display: "flex", alignItems: "center",
      padding: "0 16px",
      fontSize: 10, letterSpacing: 1.5, color: appColors.dim,
      textTransform: "uppercase",
    }}>
      <span style={{ color: appColors.faint, marginRight: 6 }}>↑↓</span>
      select
      <span style={{ color: appColors.faint, margin: "0 6px" }}>·</span>
      <span style={{ color: appColors.faint, marginRight: 6 }}>Enter</span>
      focus
      <span style={{ color: appColors.faint, margin: "0 6px" }}>·</span>
      shell to manage
    </div>
  </div>
);

const BCPane = ({ state = "two-caves" }) => {
  const isEmpty = state === "empty";
  const isWipe  = state === "wipe";

  // sample caves
  const caves = isEmpty ? [] : [
    { badge: "RUN", name: "kernel",      type: "PERS", caps: ["NET","DSP","FS"], st: "ok"   },
    { badge: "RUN", name: "research-01", type: "EPHM", caps: ["NET","RAW"],      st: "ok"   },
  ];
  const wipeCave = isWipe
    ? { badge: "WPE", name: "research-01", type: "EPHM", caps: ["NET","RAW"], st: "warn" }
    : null;

  const allCaves = isWipe ? [caves[0], wipeCave] : caves;
  const selectedIdx = isWipe ? 1 : 0;
  const selected = allCaves[selectedIdx];

  // detail panel data
  let detail = null;
  if (selected) {
    if (isWipe) {
      detail = {
        name: "research-01",
        state: "warn",
        type: "EPHM",
        fsKey: "—",
        wipe: true,
        caps: ["NET","RAW"],
        tools: 4,
        audit: 28,
        created: "0d 14m ago",
        auditLog: [
          { idx: 251, cat: "cave  :", text: "destroy initiated · research-01" },
          { idx: 250, cat: "cave  :", text: "seal · persistent → ephemeral" },
          { idx: 249, cat: "cave  :", text: "exit · attach session 0d 12m" },
          { idx: 248, cat: "cave  :", text: "enter · 0d 12m ago" },
        ],
      };
    } else if (selected.name === "kernel") {
      detail = {
        name: "kernel",
        state: "ok",
        type: "PERS",
        fsKey: "c4e3d7a2",
        wipe: false,
        caps: ["NET","DSP","FS"],
        tools: 0,
        audit: 247,
        created: "0d 14m ago",
        auditLog: [
          { idx: 247, cat: "cave  :", text: "audit category Cave registered" },
          { idx: 220, cat: "cave  :", text: "grant fs · kernel" },
          { idx: 219, cat: "cave  :", text: "grant dsp · kernel" },
          { idx: 218, cat: "cave  :", text: "created · kernel · 0d 14m ago" },
        ],
      };
    }
  }

  return (
    <div style={{
      width: "100%", height: "100%", background: appColors.bg,
      display: "flex", flexDirection: "column",
      fontFamily: appMono, overflow: "hidden",
    }}>
      {/* HEADER */}
      <Strip height={32} bottom>
        <StripSeg padL={16} separator={false}>
          <span style={{
            fontSize: 12, fontWeight: 700, letterSpacing: 2, color: appColors.ink,
          }}>
            CAVES
          </span>
          <span style={{ color: appColors.faint, marginLeft: 12, letterSpacing: 1 }}>
            Isolated container runtime
          </span>
        </StripSeg>
        <div style={{ flex: 1 }} />
        <StripSeg separator={false} padR={16} color={appColors.dim}>
          <span style={{ color: appColors.ink, fontVariantNumeric: "tabular-nums" }}>{allCaves.length}</span>
          <span>/ 32 SLOTS</span>
        </StripSeg>
      </Strip>

      {/* BODY · split */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* LEFT: table */}
        <div style={{ flex: "0 0 60%", display: "flex", flexDirection: "column", borderRight: `1px solid ${appColors.hair}`, overflow: "hidden" }}>
          {!isEmpty && <BCTableHeader />}
          <div style={{ flex: 1, overflow: "hidden" }}>
            {isEmpty ? (
              <div style={{
                height: "100%", display: "flex", alignItems: "center", justifyContent: "center",
                color: appColors.dim, fontSize: 12, letterSpacing: 1,
                padding: 24, textAlign: "center",
              }}>
                (no Caves — use <span style={{ color: appColors.cyan, margin: "0 6px" }}>caves create &lt;name&gt;</span> in shell)
              </div>
            ) : allCaves.map((c, i) => (
              <BCCaveRow
                key={c.name}
                state={c.st}
                badge={c.badge}
                name={c.name}
                type={c.type}
                caps={c.caps}
                selected={i === selectedIdx}
              />
            ))}
          </div>
        </div>

        {/* RIGHT: detail */}
        <div style={{ flex: "1 1 40%", display: "flex", flexDirection: "column", overflow: "hidden" }}>
          {detail ? <BCDetailCave cave={detail} /> : <BCDetailEmpty />}
        </div>
      </div>

      {/* BOTTOM */}
      <BCBottomStrip
        count={allCaves.length}
        max={32}
        running={isEmpty ? 0 : (isWipe ? 1 : 2)}
        stopped={0}
        deleted={isWipe ? 1 : 0}
      />
    </div>
  );
};

Object.assign(window, { WBPane, BCPane });
