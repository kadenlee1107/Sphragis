// Spec for FS · ED · CM (Wave 3).

const w3Colors = {
  bg: "#0A0A0A", panel: "#0E0E0E",
  hair: "#1A1A1A", hairHi: "#262626",
  ink: "#E5E7EB", mid: "#9CA3AF", dim: "#4B5563", faint: "#374151",
  cyan: "#22D3EE", green: "#22C55E", amber: "#F59E0B", red: "#EF4444",
};
const w3Mono = `"JetBrains Mono", "IBM Plex Mono", "SF Mono", Menlo, monospace`;

const W3Row = ({ k, v }) => (
  <div style={{
    display: "flex", justifyContent: "space-between", gap: 16,
    padding: "6px 0",
    borderBottom: `1px solid ${w3Colors.hair}`,
    fontSize: 11, letterSpacing: 1,
  }}>
    <span style={{ color: w3Colors.dim, textTransform: "uppercase", flexShrink: 0 }}>{k}</span>
    <span style={{ color: w3Colors.ink, textAlign: "right" }}>{v}</span>
  </div>
);

const W3Block = ({ title, children }) => (
  <div style={{
    border: `1px solid ${w3Colors.hair}`, background: "#0E0E0E",
    padding: 16,
  }}>
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w3Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>{title}</div>
    {children}
  </div>
);

const Wave3Specs = () => (
  <div style={{
    width: 1280, padding: 56, background: w3Colors.bg,
    color: w3Colors.ink, fontFamily: w3Mono,
  }}>
    <div style={{
      fontSize: 11, letterSpacing: 3, color: w3Colors.cyan,
      textTransform: "uppercase", marginBottom: 8,
    }}>
      [spec] sphragis · fs · ed · cm · v0.5.0-dev
    </div>
    <div style={{ fontSize: 24, letterSpacing: 2, marginBottom: 4 }}>
      Implementation reference (wave 3)
    </div>
    <div style={{ fontSize: 12, color: w3Colors.dim, marginBottom: 32 }}>
      Pane 1280×748. Header strip 32px (FS, CM) or 24px tab bar (ED). Bottom strip 28px.
    </div>

    {/* SHARED */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w3Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>shared strip rhythm</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 32 }}>
      <W3Block title="Strips">
        <W3Row k="Header (FS, CM)" v="32px · #0A0A0A · 1px #1A1A1A bottom" />
        <W3Row k="Tab bar (ED)"     v="24px · same as desktop tab strip" />
        <W3Row k="Body"             v="fills remaining height" />
        <W3Row k="Bottom"           v="28px · 1px #1A1A1A top · segments separated 1px #262626" />
        <W3Row k="Padding"          v="12-16px L/R per segment · 8px gap inside segment" />
        <W3Row k="Conn pill"        v="reuses desktop status-pill: 1px #262626 border, panel bg, 6px dot + 1px ring + label" />
      </W3Block>
      <W3Block title="Empty-state typography (all apps)">
        <W3Row k="Style"  v="centered · 12px · #4B5563 dim · letter-spacing 1px" />
        <W3Row k="Inline cmd hint" v="cyan #22D3EE · monospace, no border" />
        <W3Row k="FS"  v="(vault is empty — use 'write &lt;name&gt; &lt;data&gt;' in shell)" />
        <W3Row k="CM"  v="(no peer connected — use 'comms connect &lt;ip&gt;:&lt;port&gt;' in shell)" />
        <W3Row k="ED"  v="N/A — empty buffer would say '(empty file)' but all 3 sample tabs have content" />
      </W3Block>
    </div>

    {/* FS */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w3Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>fs · files (data table)</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 32 }}>
      <W3Block title="Table columns (1280px wide · 16px L/R margin)">
        <W3Row k="Inner width" v="1248px (after 16px L/R)" />
        <W3Row k="STATUS"   v="x=16  · w=120 · '[ENC]' green / '[RAW]' amber + 6px dot + 1px ring" />
        <W3Row k="FILENAME" v="x=136 · flex · ink · 'f' prefix dim · ellipsis on overflow" />
        <W3Row k="SIZE"     v="x=N   · w=120 · tabular-nums · right-aligned · unit dim" />
        <W3Row k="CIPHER"   v="w=160 · 'AES-256-CTR' cyan / '—' dim" />
        <W3Row k="MERKLE OK" v="w=110 · 'OK ✓' green · letter-spacing 1" />
        <W3Row k="Header row" v="24px · 9px / tracking 1.5 / faint · upper · 1px hair below" />
        <W3Row k="Data row"   v="24px · 1px hair below · 12px ink" />
      </W3Block>
      <W3Block title="Selection">
        <W3Row k="Border" v="1px inset #0E7490 cyanDim around the whole row" />
        <W3Row k="Underline" v="2px cyan #22D3EE strip at row bottom · inset 16px L/R" />
        <W3Row k="Background" v="UNCHANGED — row stays bg #0A0A0A" />
        <W3Row k="Alt rows" v="DISABLED in this design — table reads cleaner without zebra; revisit if density rises" />
        <W3Row k="Single-select only" v="multi-select is not in scope" />
      </W3Block>
    </div>

    {/* ED */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w3Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>ed · editor</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <W3Block title="Tab bar (24px tall)">
        <W3Row k="Tab width" v="168px · 1px #1A1A1A right border per tab" />
        <W3Row k="Active filename" v="11px ink #E5E7EB · letter-spacing 0.5" />
        <W3Row k="Inactive filename" v="11px dim #4B5563" />
        <W3Row k="Dirty marker" v="amber '•' after name (no separate column)" />
        <W3Row k="Close glyph" v="active: dim ':' + cyan 'x' · inactive: faint" />
        <W3Row k="Active underline" v="2px cyan #22D3EE · full tab width · flush bottom" />
        <W3Row k="'+' new tab" v="32px slot at far right · 1px #1A1A1A left border · 14px dim" />
      </W3Block>
      <W3Block title="Gutter + code">
        <W3Row k="Gutter width" v="56px · bg #080808 · 1px #262626 right border" />
        <W3Row k="Line numbers" v="11px · tabular · #3A3A3A · right-aligned · 12px right padding" />
        <W3Row k="Current line num" v="ink #E5E7EB" />
        <W3Row k="Current line accent" v="1px cyan #22D3EE · vertical · at gutter's right edge · 16px tall" />
        <W3Row k="Current line bg" v="rgba(34,211,238,0.04) — barely there" />
        <W3Row k="Code padding" v="8px T/B · 16px L/R" />
        <W3Row k="Line height" v="16px (matches 8×16 bitmap font)" />
      </W3Block>
    </div>
    <div style={{ marginBottom: 32 }}>
      <W3Block title="Syntax token → color (Rust)">
        <W3Row k="KEYWORD" v="cyan  #22D3EE · use, fn, mod, pub, let, if, const, static, unsafe, extern, loop, crate" />
        <W3Row k="STRING"  v={`green #22C55E · '...' and "..."`} />
        <W3Row k="COMMENT" v="faint #374151 · // and //! and ///" />
        <W3Row k="ATTR"    v="amber #F59E0B · #![...] and #[...]" />
        <W3Row k="IDENT"   v="ink   #E5E7EB · default" />
        <W3Row k="PUNCT"   v="mid   #9CA3AF · {}() ;,::" />
        <W3Row k="Cursor"  v="8×14 cyan block · 1Hz blink · placeholder (no input wired)" />
        <W3Row k="Status 'READ ONLY'" v="amber #F59E0B · 10px / tracking 1.5 / 500 weight · right of status bar" />
      </W3Block>
    </div>

    {/* CM */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w3Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>cm · comms</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <W3Block title="Header (32px)">
        <W3Row k="Wordmark" v="'COMMS' 12px / 700 / tracking 2 · ink · 16px L padding" />
        <W3Row k="Conn pill" v="DISCONNECTED red · CONNECTING amber · CONNECTED green + 'peer 10.0.2.42:9100' value" />
        <W3Row k="Right (CONN only)" v="cipher pill 'AES-256-CTR' cyan · key pill 'K c4e3d7a2…' (8 hex chars + dim ellipsis)" />
        <W3Row k="Pill spacing" v="8px between pills · 16px R padding" />
      </W3Block>
      <W3Block title="Message row geometry">
        <W3Row k="Row height" v="18px · 12px font" />
        <W3Row k="Grid cols"  v="60px · 32px · 56px · flex" />
        <W3Row k="Timestamp"  v="60px · '[HH:MM]' · dim · tabular-nums" />
        <W3Row k="Direction"  v="32px · '&gt;&gt;' cyan (out) / '&lt;&lt;' green (in) · 700" />
        <W3Row k="Sender"     v="56px · 'you' cyan / 'peer' green · letter-spacing 1" />
        <W3Row k="Body"       v="flex · ink · soft-wrap · continuation row repeats grid (cols 1-3 empty)" />
        <W3Row k="System msg" v="grid 60px · flex · text mid #9CA3AF prefixed '· ' · no sender" />
      </W3Block>
    </div>
    <div style={{ marginBottom: 32 }}>
      <W3Block title="Composer (28px)">
        <W3Row k="Background"  v="#0E0E0E panel — distinguishes from chat body" />
        <W3Row k="Top border"  v="1px #1A1A1A" />
        <W3Row k="Prompt"      v="'&gt;' cyan #22D3EE · 700 · 16px L padding" />
        <W3Row k="Typed text"  v="12px ink #E5E7EB" />
        <W3Row k="Cursor"      v="8×14 cyan block · 1Hz blink · trails text" />
        <W3Row k="Char counter" v="'N / 80' · tracking 1.5" />
        <W3Row k="Counter color" v="dim N&lt;70 · amber 70-79 · red ≥80 (max)" />
        <W3Row k="Disabled state" v="prompt faint #374151 · placeholder text faint · counter shows '0 / 80'" />
      </W3Block>
    </div>

    {/* SUBSTITUTIONS */}
    <W3Block title="Substitutions (do not fake)">
      <W3Row k="Editor" v="READ ONLY (amber) — cursor is decorative, no typing wired" />
      <W3Row k="FS" v="no perms / owner / mtime — only STATUS / NAME / SIZE / CIPHER / MERKLE columns" />
      <W3Row k="CM key exchange" v="X25519 is placeholder; session key derived SHA-256(peer_ip + ts) — pill labeled 'K' not 'X25519'" />
      <W3Row k="CM message length" v="80-char hard cap shown by counter — enforced client-side" />
      <W3Row k="FS tree view" v="not implemented — SealFS is currently flat" />
    </W3Block>
  </div>
);

window.Wave3Specs = Wave3Specs;
