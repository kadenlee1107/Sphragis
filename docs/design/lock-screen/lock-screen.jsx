// Sphragis lock screen — single artboard renderer, accepts a `state` prop.
// state: "idle" | "typing" | "denied"

const lockColors = {
  bg: "#0A0A0A",
  panel: "#0E0E0E",
  hairline: "#1A1A1A",
  hairlineHi: "#262626",
  textHi: "#E5E7EB",
  textMid: "#9CA3AF",
  textDim: "#4B5563",
  textFaint: "#374151",
  cyan: "#22D3EE",
  cyanDim: "#0E7490",
  cyanGlow: "#0E3A44",
  green: "#22C55E",
  greenDim: "#14532D",
  amber: "#F59E0B",
  amberDim: "#78350F",
  red: "#EF4444",
  redDim: "#7F1D1D",
};

const mono = `"JetBrains Mono", "IBM Plex Mono", "SF Mono", Menlo, monospace`;

// — Subcomponents ————————————————————————————————————————————

const StatusPill = ({ label, tone = "green", value }) => {
  const c =
    tone === "green" ? lockColors.green :
    tone === "amber" ? lockColors.amber :
    tone === "red"   ? lockColors.red   :
    lockColors.cyan;
  const dim =
    tone === "green" ? lockColors.greenDim :
    tone === "amber" ? lockColors.amberDim :
    tone === "red"   ? lockColors.redDim   :
    lockColors.cyanDim;
  return (
    <div style={{
      display: "inline-flex", alignItems: "center", gap: 8,
      padding: "4px 10px",
      border: `1px solid ${lockColors.hairlineHi}`,
      background: lockColors.panel,
      fontFamily: mono, fontSize: 11, letterSpacing: 1.2,
      color: lockColors.textMid, textTransform: "uppercase",
    }}>
      <span style={{
        width: 6, height: 6, background: c,
        boxShadow: `0 0 0 1px ${dim}`,
        display: "inline-block",
      }} />
      <span style={{ color: lockColors.textHi, fontWeight: 500 }}>{label}</span>
      {value && (
        <span style={{ color: lockColors.textDim, marginLeft: 4 }}>{value}</span>
      )}
    </div>
  );
};

const Bracket = ({ children, color = lockColors.cyan }) => (
  <span style={{ fontFamily: mono, color: lockColors.textDim }}>
    <span style={{ color }}>[</span>
    {children}
    <span style={{ color }}>]</span>
  </span>
);

// — Main ————————————————————————————————————————————————————

const LockScreen = ({ state = "idle", width = 1280, height = 800 }) => {
  const isIdle = state === "idle";
  const isTyping = state === "typing";
  const isDenied = state === "denied";

  const dotsCount = isTyping ? 7 : isDenied ? 7 : 0;
  const attempts = isDenied ? 3 : 4;

  const accent = isDenied ? lockColors.red : lockColors.cyan;
  const accentDim = isDenied ? lockColors.redDim : lockColors.cyanDim;

  // Boot log lines (last 4)
  const bootLog = [
    { tag: "ok", text: "[net] virtio-net up  10.0.0.42/24" },
    { tag: "ok", text: "[fs]  sealfs mounted /  ro  aes-xts-512" },
    { tag: "ok", text: "[sec] tpm seal verified  pcr0..7 match" },
    { tag: "ok", text: "[ui]  framebuffer 1280x800 bgra8" },
  ];

  // Crosshair corner mark
  const Corner = ({ pos }) => {
    const s = 14;
    const t = {
      tl: { top: 24, left: 24, transform: "rotate(0deg)" },
      tr: { top: 24, right: 24, transform: "rotate(90deg)" },
      bl: { bottom: 24, left: 24, transform: "rotate(270deg)" },
      br: { bottom: 24, right: 24, transform: "rotate(180deg)" },
    }[pos];
    return (
      <div style={{ position: "absolute", width: s, height: s, ...t }}>
        <div style={{ position: "absolute", left: 0, top: 0, width: s, height: 1, background: lockColors.hairlineHi }} />
        <div style={{ position: "absolute", left: 0, top: 0, width: 1, height: s, background: lockColors.hairlineHi }} />
      </div>
    );
  };

  return (
    <div style={{
      width, height, position: "relative",
      background: lockColors.bg,
      color: lockColors.textHi,
      fontFamily: mono,
      overflow: "hidden",
      // subtle scanline (under 5% of screen if you slice it; here it's a 1px repeat — cheap)
      backgroundImage: `repeating-linear-gradient(0deg, rgba(255,255,255,0.012) 0 1px, transparent 1px 3px)`,
    }}>
      {/* Crosshair markers */}
      <Corner pos="tl" />
      <Corner pos="tr" />
      <Corner pos="bl" />
      <Corner pos="br" />

      {/* — TOP STATUS ROW ————————————————————————————— */}
      <div style={{
        position: "absolute", top: 24, left: 56, right: 56,
        display: "flex", alignItems: "center", justifyContent: "space-between",
        height: 28,
      }}>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <StatusPill label="ENCRYPTED" tone="green" value="AES-XTS-512" />
          <StatusPill label="SECURE BOOT" tone="green" value="OK" />
          <StatusPill label="TPM" tone="green" value="SEALED" />
          <StatusPill label="NET" tone={isDenied ? "amber" : "green"} value={isDenied ? "ISOLATED" : "10.0.0.42"} />
        </div>
        <div style={{
          display: "flex", gap: 16, alignItems: "center",
          fontSize: 11, color: lockColors.textMid, letterSpacing: 1.2, textTransform: "uppercase",
        }}>
          <span>HOST <span style={{ color: lockColors.textHi }}>nyx-01</span></span>
          <span>KERNEL <span style={{ color: lockColors.textHi }}>bat 0.4.2-rc1</span></span>
          <span>ARCH <span style={{ color: lockColors.textHi }}>aarch64 / m-series</span></span>
        </div>
      </div>

      {/* hairline under status row */}
      <div style={{
        position: "absolute", top: 64, left: 56, right: 56, height: 1,
        background: lockColors.hairline,
      }} />

      {/* — CENTER STACK ——————————————————————————————— */}
      <div style={{
        position: "absolute", left: "50%", top: "50%",
        transform: "translate(-50%, -50%)",
        display: "flex", flexDirection: "column", alignItems: "center",
        width: 560,
      }}>
        {/* Project glyph */}
        <div style={{ marginBottom: 24, position: "relative" }}>
          <BatGlyph size={96} stroke={accent} node={accent} dim={isDenied ? lockColors.redDim : lockColors.cyanGlow} />
        </div>

        {/* Wordmark */}
        <div style={{
          fontFamily: mono, fontWeight: 700, fontSize: 32,
          letterSpacing: 8, color: lockColors.textHi,
          marginBottom: 6,
        }}>
          BAT<span style={{ color: accent }}>_</span>OS
        </div>
        <div style={{
          fontFamily: mono, fontSize: 11,
          letterSpacing: 3, color: lockColors.textDim,
          marginBottom: 40, textTransform: "uppercase",
        }}>
          v0.4.2-rc1 &nbsp;·&nbsp; build 20260428.a3f1c &nbsp;·&nbsp; signed
        </div>

        {/* Field label */}
        <div style={{
          alignSelf: "stretch",
          display: "flex", justifyContent: "space-between",
          fontFamily: mono, fontSize: 10, letterSpacing: 2,
          color: lockColors.textMid, textTransform: "uppercase",
          marginBottom: 8, padding: "0 4px",
        }}>
          <span>
            <Bracket color={accent}>auth</Bracket>
            &nbsp;<span style={{ color: lockColors.textHi }}>passphrase</span>
          </span>
          <span style={{ color: lockColors.textDim }}>argon2id · 64MB · t=3</span>
        </div>

        {/* Passphrase field */}
        <div style={{
          alignSelf: "stretch",
          height: 56,
          border: `1px solid ${isDenied ? lockColors.red : (isTyping ? accent : lockColors.hairlineHi)}`,
          background: lockColors.panel,
          display: "flex", alignItems: "center",
          padding: "0 18px",
          position: "relative",
          boxShadow: isTyping ? `0 0 0 1px ${accentDim} inset` : "none",
        }}>
          {/* prompt prefix */}
          <span style={{
            color: accent, fontFamily: mono, fontSize: 18, fontWeight: 500,
            marginRight: 14, userSelect: "none",
          }}>
            ▌
          </span>
          {/* dots */}
          <div style={{
            display: "flex", alignItems: "center", gap: 8,
            flex: 1,
          }}>
            {Array.from({ length: dotsCount }).map((_, i) => (
              <span key={i} style={{
                width: 8, height: 8, background: lockColors.textHi,
                display: "inline-block",
              }} />
            ))}
            {/* cursor */}
            {!isDenied && (
              <span style={{
                width: 10, height: 22,
                background: isTyping ? accent : lockColors.textMid,
                marginLeft: dotsCount > 0 ? 4 : 0,
                animation: "batCursor 1s steps(2) infinite",
                display: "inline-block",
              }} />
            )}
          </div>
          {/* attempts inline */}
          <span style={{
            fontFamily: mono, fontSize: 10, letterSpacing: 1.5,
            color: isDenied ? lockColors.red : lockColors.textDim,
            textTransform: "uppercase",
          }}>
            {attempts} attempts left
          </span>
        </div>

        {/* helper row */}
        <div style={{
          alignSelf: "stretch",
          display: "flex", justifyContent: "space-between",
          fontFamily: mono, fontSize: 10, letterSpacing: 1.5,
          color: lockColors.textDim, textTransform: "uppercase",
          marginTop: 10, padding: "0 4px",
        }}>
          <span>RETURN to submit · ESC to wipe · F2 keymap</span>
          <span>caps off</span>
        </div>

        {/* — DENIED OVERLAY — only on denied state — */}
        {isDenied && (
          <div style={{
            position: "absolute", left: "50%", top: "50%",
            transform: "translate(-50%, -50%)",
            zIndex: 5,
            border: `1px solid ${lockColors.red}`,
            background: "#0A0A0A",
            padding: "28px 56px",
            textAlign: "center",
            boxShadow: `0 0 0 1px ${lockColors.redDim}`,
          }}>
            <div style={{
              fontFamily: mono, fontWeight: 700, fontSize: 28,
              letterSpacing: 8, color: lockColors.red,
            }}>
              ACCESS DENIED
            </div>
            <div style={{
              fontFamily: mono, fontSize: 10, letterSpacing: 2,
              color: lockColors.textMid, textTransform: "uppercase",
              marginTop: 10,
            }}>
              code 0x1A · argon2id verify failed · 1.42s
            </div>
            <div style={{
              fontFamily: mono, fontSize: 10, letterSpacing: 2,
              color: lockColors.red, textTransform: "uppercase",
              marginTop: 4,
            }}>
              attempt 2 of 6 · cooldown 8s
            </div>
          </div>
        )}
      </div>

      {/* — BOTTOM-LEFT BOOT LOG — */}
      <div style={{
        position: "absolute", bottom: 24, left: 56,
        fontFamily: mono, fontSize: 11, lineHeight: "16px",
        color: lockColors.textDim, width: 460,
      }}>
        <div style={{
          fontSize: 10, letterSpacing: 2, color: lockColors.textFaint,
          textTransform: "uppercase", marginBottom: 6,
          display: "flex", justifyContent: "space-between",
        }}>
          <span>boot.log · tail -n 4</span>
          <span style={{ color: lockColors.textDim }}>2.41s to ready</span>
        </div>
        {bootLog.map((line, i) => (
          <div key={i} style={{ display: "flex", gap: 10 }}>
            <span style={{ color: lockColors.green }}>[ ok ]</span>
            <span>{line.text}</span>
          </div>
        ))}
      </div>

      {/* — BOTTOM-RIGHT TIMESTAMP + ATTEMPTS — */}
      <div style={{
        position: "absolute", bottom: 24, right: 56,
        textAlign: "right",
        fontFamily: mono, fontSize: 11, lineHeight: "16px",
      }}>
        <div style={{
          fontSize: 10, letterSpacing: 2, color: lockColors.textFaint,
          textTransform: "uppercase", marginBottom: 6,
        }}>
          system clock · utc
        </div>
        <div style={{ color: lockColors.textHi, fontSize: 14, letterSpacing: 2 }}>
          2026-05-02 &nbsp; 14:22:08
        </div>
        <div style={{ color: lockColors.textDim, marginTop: 4, letterSpacing: 1.5 }}>
          uptime 0d 00:02:41
        </div>
        <div style={{
          marginTop: 10,
          display: "inline-flex", alignItems: "center", gap: 8,
          padding: "4px 10px",
          border: `1px solid ${isDenied ? lockColors.red : lockColors.hairlineHi}`,
          background: lockColors.panel,
          fontSize: 11, letterSpacing: 1.5, textTransform: "uppercase",
          color: isDenied ? lockColors.red : lockColors.textMid,
        }}>
          <span style={{
            width: 6, height: 6,
            background: isDenied ? lockColors.red : lockColors.amber,
            boxShadow: `0 0 0 1px ${isDenied ? lockColors.redDim : lockColors.amberDim}`,
            display: "inline-block",
          }} />
          {attempts} attempts remaining
        </div>
      </div>

      {/* — BOTTOM EDGE TICKER — */}
      <div style={{
        position: "absolute", bottom: 0, left: 0, right: 0,
        height: 1, background: lockColors.hairline,
      }} />
    </div>
  );
};

window.LockScreen = LockScreen;
