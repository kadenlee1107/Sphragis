// FS · ED · CM — three interactive apps for Bat_OS Wave 3.

// — Shared strip primitives ————————————————————————————————

const Strip = ({ height, top, bottom, children }) => (
  <div style={{
    height, flexShrink: 0,
    background: appColors.bg,
    borderTop: top ? `1px solid ${appColors.hair}` : "none",
    borderBottom: bottom ? `1px solid ${appColors.hair}` : "none",
    display: "flex", alignItems: "stretch",
    fontFamily: appMono,
  }}>
    {children}
  </div>
);

const StripSeg = ({ children, separator = true, padL = 12, padR = 12, color = appColors.mid, flex }) => (
  <div style={{
    display: "flex", alignItems: "center", gap: 8,
    padding: `0 ${padR}px 0 ${padL}px`,
    borderRight: separator ? `1px solid ${appColors.hairHi}` : "none",
    fontSize: 10, letterSpacing: 1.5, color,
    textTransform: "uppercase", flex,
  }}>
    {children}
  </div>
);

const ConnPill = ({ tone, label, value }) => {
  const c =
    tone === "ok"   ? appColors.green :
    tone === "warn" ? appColors.amber :
    tone === "fail" ? appColors.red   :
                      appColors.cyan;
  const dim =
    tone === "ok"   ? appColors.greenDim :
    tone === "warn" ? appColors.amberDim :
    tone === "fail" ? appColors.redDim   :
                      appColors.cyanDim;
  return (
    <div style={{
      display: "inline-flex", alignItems: "center", gap: 8,
      padding: "4px 10px",
      border: `1px solid ${appColors.hairHi}`,
      background: appColors.panel,
      fontSize: 10, letterSpacing: 1.5,
      color: appColors.mid, textTransform: "uppercase",
    }}>
      <span style={{
        width: 6, height: 6, background: c,
        boxShadow: `0 0 0 1px ${dim}`,
        display: "inline-block",
      }} />
      <span style={{ color: appColors.ink, fontWeight: 500 }}>{label}</span>
      {value && <span style={{ color: appColors.dim, marginLeft: 4 }}>{value}</span>}
    </div>
  );
};

// — FS · FILES ————————————————————————————————————————————

const FSRow = ({ enc, name, size, unit, selected }) => {
  const tagColor = enc ? appColors.green : appColors.amber;
  const tagText  = enc ? "[ENC]" : "[RAW]";
  const tagRing  = enc ? appColors.greenDim : appColors.amberDim;
  return (
    <div style={{ position: "relative" }}>
      <div style={{
        display: "grid",
        gridTemplateColumns: "120px 1fr 120px 160px 110px",
        height: 24, alignItems: "center",
        fontSize: 12, color: appColors.ink,
        padding: "0 16px",
        border: selected ? `1px solid ${appColors.cyanDim}` : "1px solid transparent",
        borderBottom: `1px solid ${selected ? appColors.cyanDim : appColors.hair}`,
      }}>
        <span style={{ display: "inline-flex", alignItems: "center", gap: 8, color: tagColor, fontSize: 10, letterSpacing: 1 }}>
          <span style={{
            width: 6, height: 6, background: tagColor,
            boxShadow: `0 0 0 1px ${tagRing}`, display: "inline-block",
          }} />
          {tagText}
        </span>
        <span style={{ color: appColors.ink, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          <span style={{ color: appColors.dim, marginRight: 8 }}>f</span>
          {name}
        </span>
        <span style={{ textAlign: "right", paddingRight: 16, fontVariantNumeric: "tabular-nums", color: appColors.ink }}>
          {size}<span style={{ color: appColors.dim, marginLeft: 4 }}>{unit}</span>
        </span>
        <span style={{ paddingLeft: 4, color: enc ? appColors.cyan : appColors.dim }}>
          {enc ? "AES-256-CTR" : "—"}
        </span>
        <span style={{ color: appColors.green, letterSpacing: 1 }}>OK ✓</span>
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

const FSHeader = () => (
  <div style={{
    display: "grid",
    gridTemplateColumns: "120px 1fr 120px 160px 110px",
    height: 24, alignItems: "center",
    padding: "0 16px",
    fontSize: 9, letterSpacing: 1.5, color: appColors.faint,
    textTransform: "uppercase",
    borderBottom: `1px solid ${appColors.hair}`,
  }}>
    <span>STATUS</span>
    <span>FILENAME</span>
    <span style={{ textAlign: "right", paddingRight: 16 }}>SIZE</span>
    <span style={{ paddingLeft: 4 }}>CIPHER</span>
    <span>MERKLE OK</span>
  </div>
);

const FSPane = ({ state = "populated" }) => {
  const populated = [
    { enc: true,  name: "kernel_main.rs", size: "12.4", unit: "KiB" },
    { enc: true,  name: "boot.log",        size: "4 218", unit: "B" },
    { enc: true,  name: "audit.ring",      size: "1.0",  unit: "MiB" },
    { enc: true,  name: "pinset.toml",     size: "812",  unit: "B" },
    { enc: false, name: "scratch.txt",     size: "208",  unit: "B" },
    { enc: true,  name: "research-01.cave",size: "184",  unit: "KiB" },
  ];
  const selectedIdx = state === "large-selected" ? 5 : 1;
  const isEmpty = state === "empty";
  const fileCount = isEmpty ? 0 : populated.length;
  return (
    <div style={{
      width: "100%", height: "100%", background: appColors.bg,
      display: "flex", flexDirection: "column",
      fontFamily: appMono, overflow: "hidden",
    }}>
      {/* HEADER */}
      <Strip height={32} bottom>
        <StripSeg color={appColors.ink} padL={16}>
          <span style={{ color: appColors.faint }}>VAULT</span>
          <span style={{ color: appColors.ink, letterSpacing: 1 }}>ENCRYPTED</span>
          <span style={{ color: appColors.faint }}>·</span>
          <span style={{ color: appColors.cyan }}>AES-256-CTR</span>
          <span style={{ color: appColors.faint }}>+ SHA-256 integrity</span>
        </StripSeg>
        <div style={{ flex: 1 }} />
        <StripSeg separator={false} color={appColors.dim} padR={16}>
          <span style={{ color: appColors.ink, fontVariantNumeric: "tabular-nums" }}>{fileCount}</span>
          <span>/ MAX_FILES 32</span>
        </StripSeg>
      </Strip>

      {/* TABLE */}
      <div style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column" }}>
        {!isEmpty && <FSHeader />}
        {isEmpty ? (
          <div style={{
            flex: 1, display: "flex", alignItems: "center", justifyContent: "center",
            color: appColors.dim, fontSize: 12, letterSpacing: 1,
          }}>
            (vault is empty — use <span style={{ color: appColors.cyan, margin: "0 6px" }}>write &lt;name&gt; &lt;data&gt;</span> in shell)
          </div>
        ) : (
          <div>
            {populated.map((f, i) => (
              <FSRow key={f.name} {...f} selected={i === selectedIdx} />
            ))}
          </div>
        )}
      </div>

      {/* FOOTER */}
      <Strip height={28} top>
        <StripSeg padL={16} color={appColors.mid}>
          <span style={{ color: appColors.faint }}>FILES</span>
          <span style={{ color: appColors.ink, fontVariantNumeric: "tabular-nums" }}>{fileCount}</span>
          <span>·</span>
          <span style={{ color: appColors.faint }}>MAX_FILES</span>
          <span style={{ color: appColors.ink }}>32</span>
        </StripSeg>
        <StripSeg color={appColors.mid}>
          <span style={{ color: appColors.faint }}>MERKLE</span>
          <span style={{ color: appColors.ink, fontVariantNumeric: "tabular-nums" }}>c4e3 d7a2</span>
          <span style={{ color: appColors.dim }}>…</span>
          <span style={{ color: appColors.green }}>VERIFIED</span>
        </StripSeg>
        <div style={{ flex: 1 }} />
        <StripSeg separator={false} padR={16} color={appColors.dim}>
          <span>Ctrl+1 to manage in shell</span>
        </StripSeg>
      </Strip>
    </div>
  );
};

// — ED · EDITOR ——————————————————————————————————————————

const EDTab = ({ name, active, dirty }) => (
  <div style={{
    position: "relative",
    width: 168, height: "100%",
    display: "flex", alignItems: "center", justifyContent: "space-between",
    padding: "0 12px",
    borderRight: `1px solid ${appColors.hair}`,
    background: active ? appColors.bg : "transparent",
  }}>
    <span style={{
      fontSize: 11, color: active ? appColors.ink : appColors.dim,
      letterSpacing: 0.5,
    }}>
      {name}
      {dirty && <span style={{ color: appColors.amber, marginLeft: 4 }}>•</span>}
    </span>
    <span style={{
      fontSize: 11, color: active ? appColors.cyan : appColors.faint,
    }}>
      <span style={{ color: appColors.faint }}>:</span>x
    </span>
    {active && (
      <div style={{
        position: "absolute", left: 0, right: 0, bottom: 0, height: 2,
        background: appColors.cyan,
      }} />
    )}
  </div>
);

// Tokenized line — color rules drive text spans
const tok = {
  k: appColors.cyan,    // keyword
  s: appColors.green,   // string
  c: appColors.faint,   // comment
  a: appColors.amber,   // attribute
  i: appColors.ink,     // ident
  p: appColors.mid,     // punct
};

const Span = ({ k, children }) => <span style={{ color: tok[k] }}>{children}</span>;

const EDPane = () => {
  // Lines list — each line is array of [kind, text] pairs.
  // Drawing as JSX inline.
  const lines = [
    [['c', '//! Bat_OS bare-metal kernel entry']],
    [['c', '//! v0.5.0-DEV · aarch64-unknown-none-softfloat']],
    [],
    [['a', '#![no_std]']],
    [['a', '#![no_main]']],
    [],
    [['k', 'use'], ['p', ' '], ['i', 'core'], ['p', '::'], ['i', 'panic'], ['p', '::'], ['i', 'PanicInfo'], ['p', ';']],
    [['k', 'use'], ['p', ' '], ['i', 'crate'], ['p', '::{'], ['i', 'kernel'], ['p', ', '], ['i', 'drivers'], ['p', ', '], ['i', 'fs'], ['p', ', '], ['i', 'net'], ['p', ', '], ['i', 'ui'], ['p', '};']],
    [],
    [['c', '/// Entry point — called from boot.S after stack + MMU.']],
    [['a', '#[no_mangle]']],
    [['k', 'pub'], ['p', ' '], ['k', 'extern'], ['p', ' '], ['s', '"C"'], ['p', ' '], ['k', 'fn'], ['p', ' '], ['i', 'kernel_main'], ['p', '('], ['i', 'master_key'], ['p', ': &['], ['i', 'u8'], ['p', '; '], ['i', '32'], ['p', ']) -> '], ['i', '!'], ['p', ' {']],
    [['p', '    '], ['i', 'kernel'], ['p', '::'], ['i', 'mm'], ['p', '::'], ['i', 'init'], ['p', '();']],
    [['p', '    '], ['i', 'kernel'], ['p', '::'], ['i', 'process'], ['p', '::'], ['i', 'init'], ['p', '();']],
    [['p', '    '], ['i', 'kernel'], ['p', '::'], ['i', 'scheduler'], ['p', '::'], ['i', 'init'], ['p', '();']],
    [['p', '    '], ['i', 'kernel'], ['p', '::'], ['i', 'ipc'], ['p', '::'], ['i', 'init'], ['p', '();']],
    [['p', '    '], ['i', 'kernel'], ['p', '::'], ['i', 'arch'], ['p', '::'], ['i', 'init_exceptions'], ['p', '();']],
    [],
    [['p', '    '], ['c', '// storage + net come up after the core is alive']],
    [['p', '    '], ['i', 'fs'], ['p', '::'], ['i', 'batfs'], ['p', '::'], ['i', 'init'], ['p', '(&'], ['i', 'master_key'], ['p', ');']],
    [['p', '    '], ['i', 'drivers'], ['p', '::'], ['i', 'virtio'], ['p', '::'], ['i', 'net'], ['p', '::'], ['i', 'init'], ['p', '();']],
    [['p', '    '], ['i', 'net'], ['p', '::'], ['i', 'init'], ['p', '();']],
    [],
    [['p', '    '], ['c', '// hand off to the operator shell — never returns']],
    [['p', '    '], ['i', 'ui'], ['p', '::'], ['i', 'shell'], ['p', '::'], ['i', 'run'], ['p', '();']],
    [['p', '}']],
    [],
    [['a', '#[panic_handler]']],
    [['k', 'fn'], ['p', ' '], ['i', 'panic'], ['p', '('], ['i', '_info'], ['p', ': &'], ['i', 'PanicInfo'], ['p', ') -> '], ['i', '!'], ['p', ' { '], ['k', 'loop'], ['p', ' {} }']],
  ];

  const cursorLine = 16; // 1-indexed: kernel::arch::init_exceptions();

  return (
    <div style={{
      width: "100%", height: "100%", background: appColors.bg,
      display: "flex", flexDirection: "column",
      fontFamily: appMono, overflow: "hidden",
    }}>
      {/* TAB BAR */}
      <Strip height={24} bottom>
        <EDTab name="kernel_main.rs" active />
        <EDTab name="lib.rs" />
        <EDTab name="Cargo.toml" dirty />
        <div style={{ flex: 1 }} />
        <div style={{
          width: 32, display: "flex", alignItems: "center", justifyContent: "center",
          borderLeft: `1px solid ${appColors.hair}`,
          color: appColors.dim, fontSize: 14,
        }}>+</div>
      </Strip>

      {/* GUTTER + CODE */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* GUTTER */}
        <div style={{
          width: 56, flexShrink: 0,
          background: "#080808",
          borderRight: `1px solid ${appColors.hairHi}`,
          padding: "8px 0",
          position: "relative",
        }}>
          {lines.map((_, i) => {
            const lineNo = i + 1;
            const isCur = lineNo === cursorLine;
            return (
              <div key={i} style={{
                height: 16, lineHeight: "16px",
                paddingRight: 12, textAlign: "right",
                fontSize: 11,
                color: isCur ? appColors.ink : "#3A3A3A",
                fontVariantNumeric: "tabular-nums",
                position: "relative",
              }}>
                {lineNo}
                {isCur && (
                  <div style={{
                    position: "absolute", right: 0, top: 0, width: 1, height: 16,
                    background: appColors.cyan,
                  }} />
                )}
              </div>
            );
          })}
        </div>

        {/* CODE */}
        <div style={{ flex: 1, padding: "8px 16px", overflow: "hidden" }}>
          {lines.map((spans, i) => {
            const lineNo = i + 1;
            const isCur = lineNo === cursorLine;
            return (
              <div key={i} style={{
                height: 16, lineHeight: "16px", fontSize: 12,
                whiteSpace: "pre",
                background: isCur ? "rgba(34, 211, 238, 0.04)" : "transparent",
                position: "relative",
                fontFamily: appMono,
              }}>
                {spans.length === 0 ? "\u00A0" : spans.map(([k, t], j) => (
                  <Span key={j} k={k}>{t}</Span>
                ))}
                {isCur && (
                  <span style={{
                    position: "absolute",
                    left: `${5 * 7.2}px`, // approx col 5 in JetBrains Mono 12px
                    top: 1, width: 8, height: 14,
                    background: appColors.cyan,
                    animation: "shellCursor 1s steps(2) infinite",
                  }} />
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* STATUS STRIP */}
      <Strip height={28} top>
        <StripSeg padL={16}>
          <span style={{ color: appColors.faint }}>LANG</span>
          <span style={{ color: appColors.ink }}>RUST</span>
        </StripSeg>
        <StripSeg>
          <span style={{ color: appColors.faint }}>ENC</span>
          <span style={{ color: appColors.ink }}>UTF-8</span>
        </StripSeg>
        <StripSeg>
          <span style={{ color: appColors.faint }}>POS</span>
          <span style={{ color: appColors.ink, fontVariantNumeric: "tabular-nums" }}>
            Ln 17, Col 5
          </span>
        </StripSeg>
        <StripSeg>
          <span style={{ color: appColors.faint }}>LF</span>
          <span style={{ color: appColors.ink }}>UNIX</span>
        </StripSeg>
        <div style={{ flex: 1 }} />
        <StripSeg separator={false} padR={16}>
          <span style={{ color: appColors.amber, letterSpacing: 1.5, fontWeight: 500 }}>
            READ ONLY
          </span>
        </StripSeg>
      </Strip>
    </div>
  );
};

// — CM · COMMS ——————————————————————————————————————————

const CMMessage = ({ time, dir, sender, text, cont }) => {
  const isOut = dir === "out";
  const arrow = isOut ? ">>" : "<<";
  const arrowC = isOut ? appColors.cyan : appColors.green;
  const senderC = isOut ? appColors.cyan : appColors.green;
  return (
    <div style={{ fontSize: 12, fontFamily: appMono, marginBottom: 2 }}>
      <div style={{
        display: "grid",
        gridTemplateColumns: "60px 32px 56px 1fr",
        alignItems: "baseline", lineHeight: "18px",
      }}>
        <span style={{ color: appColors.dim, fontVariantNumeric: "tabular-nums" }}>[{time}]</span>
        <span style={{ color: arrowC, fontWeight: 700 }}>{arrow}</span>
        <span style={{ color: senderC, letterSpacing: 1 }}>{sender}</span>
        <span style={{ color: appColors.ink }}>{text}</span>
      </div>
      {cont && (
        <div style={{
          display: "grid",
          gridTemplateColumns: "60px 32px 56px 1fr",
          lineHeight: "18px",
        }}>
          <span /><span /><span />
          <span style={{ color: appColors.ink }}>{cont}</span>
        </div>
      )}
    </div>
  );
};

const CMSystem = ({ time, text }) => (
  <div style={{
    fontSize: 12, fontFamily: appMono, marginBottom: 2,
    display: "grid", gridTemplateColumns: "60px 1fr",
    lineHeight: "18px",
  }}>
    <span style={{ color: appColors.dim, fontVariantNumeric: "tabular-nums" }}>[{time}]</span>
    <span style={{ color: appColors.mid }}>· {text}</span>
  </div>
);

const CMPane = ({ state = "connected" }) => {
  const isDisco = state === "disconnected";
  const isTyping = state === "typing";

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
            COMMS
          </span>
        </StripSeg>
        <div style={{ display: "flex", alignItems: "center", paddingLeft: 4 }}>
          {isDisco && <ConnPill tone="fail" label="DISCONNECTED" />}
          {!isDisco && <ConnPill tone="ok" label="CONNECTED" value="peer 10.0.2.42:9100" />}
        </div>
        <div style={{ flex: 1 }} />
        {!isDisco && (
          <div style={{ display: "flex", alignItems: "center", gap: 8, paddingRight: 16 }}>
            <ConnPill tone="neutral" label="AES-256-CTR" />
            <ConnPill tone="neutral" label="K" value="c4e3d7a2…" />
          </div>
        )}
      </Strip>

      {/* TIMELINE */}
      <div style={{
        flex: 1, padding: "12px 16px", overflow: "hidden",
        display: "flex", flexDirection: "column", justifyContent: "flex-end",
      }}>
        {isDisco ? (
          <div style={{
            flex: 1, display: "flex", alignItems: "center", justifyContent: "center",
            color: appColors.dim, fontSize: 12,
          }}>
            (no peer connected — use <span style={{ color: appColors.cyan, margin: "0 6px" }}>comms connect &lt;ip&gt;:&lt;port&gt;</span> in shell)
          </div>
        ) : (
          <>
            <CMSystem time="00:14" text="connected · key exchange OK · cipher AES-256-CTR" />
            <CMMessage time="00:15" dir="in"  sender="peer" text="up. east-3 link is clean." />
            <CMMessage time="00:15" dir="out" sender="you"  text="perimeter clean. dropping into watch mode." />
            <CMMessage time="00:16" dir="in"  sender="peer" text="ack. tail -f /var/log/auth on east-3, shout if anything spikes." />
            <CMMessage time="00:18" dir="out" sender="you"  text="watching. ssh from 10.0.2.99 → blocked at fw, src port 41203." />
            <CMMessage
              time="00:19" dir="in" sender="peer"
              text="good. their src port pattern is the same as tuesday."
              cont="logging the 3-tuple now."
            />
            <CMMessage time="00:21" dir="out" sender="you"  text="same actor then. pinning to the audit ring." />
            <CMSystem time="00:22" text="peer disconnected" />
            <CMSystem time="00:23" text="reconnecting…" />
            <CMSystem time="00:23" text="reconnected · session resumed · key exchange OK" />
            <CMMessage time="00:24" dir="in"  sender="peer" text="back. switched uplink, sorry." />
            <CMMessage time="00:24" dir="out" sender="you"  text="np. nothing here while you were out." />
            <CMSystem time="00:27" text="key rotation due in 5m" />
          </>
        )}
      </div>

      {/* COMPOSER */}
      <div style={{
        height: 28, flexShrink: 0,
        background: appColors.panel,
        borderTop: `1px solid ${appColors.hair}`,
        display: "flex", alignItems: "center",
        padding: "0 16px", gap: 8,
        fontFamily: appMono, fontSize: 12,
      }}>
        <span style={{ color: isDisco ? appColors.faint : appColors.cyan, fontWeight: 700 }}>
          &gt;
        </span>
        {isTyping ? (
          <span style={{ color: appColors.ink, display: "flex", alignItems: "center" }}>
            logging that attempt now
            <span style={{
              display: "inline-block", width: 8, height: 14, marginLeft: 1,
              background: appColors.cyan,
              animation: "shellCursor 1s steps(2) infinite",
            }} />
          </span>
        ) : isDisco ? (
          <span style={{ color: appColors.faint, fontStyle: "normal" }}>
            (composer disabled · not connected)
          </span>
        ) : (
          <span style={{
            display: "inline-block", width: 8, height: 14,
            background: appColors.cyan,
            animation: "shellCursor 1s steps(2) infinite",
          }} />
        )}
        <div style={{ flex: 1 }} />
        <span style={{
          fontSize: 10, letterSpacing: 1.5,
          color: isTyping ? appColors.dim : appColors.faint,
          textTransform: "uppercase",
        }}>
          {isTyping ? "21" : "0"} <span style={{ color: appColors.faint }}>/ 80</span>
        </span>
      </div>
    </div>
  );
};

Object.assign(window, { FSPane, EDPane, CMPane });
