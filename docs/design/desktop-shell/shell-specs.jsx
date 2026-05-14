// Spec sheet for the Sphragis desktop chrome + shell pane.

const dsSpecColors = {
  bg: "#0A0A0A", panel: "#0E0E0E",
  hair: "#1A1A1A", hairHi: "#262626",
  ink: "#E5E7EB", mid: "#9CA3AF", dim: "#4B5563", faint: "#374151",
  cyan: "#22D3EE", green: "#22C55E", amber: "#F59E0B", red: "#EF4444",
};
const dsMono = `"JetBrains Mono", "IBM Plex Mono", "SF Mono", Menlo, monospace`;

const DSRow = ({ k, v }) => (
  <div style={{
    display: "flex", justifyContent: "space-between", gap: 16,
    padding: "6px 0",
    borderBottom: `1px solid ${dsSpecColors.hair}`,
    fontSize: 11, letterSpacing: 1,
  }}>
    <span style={{ color: dsSpecColors.dim, textTransform: "uppercase", flexShrink: 0 }}>{k}</span>
    <span style={{ color: dsSpecColors.ink, textAlign: "right" }}>{v}</span>
  </div>
);

const DSBlock = ({ title, children }) => (
  <div style={{
    border: `1px solid ${dsSpecColors.hair}`, background: "#0E0E0E",
    padding: 16,
  }}>
    <div style={{
      fontSize: 10, letterSpacing: 2, color: dsSpecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>{title}</div>
    {children}
  </div>
);

const ShellSpecs = () => (
  <div style={{
    width: 1280, padding: 56, background: dsSpecColors.bg,
    color: dsSpecColors.ink, fontFamily: dsMono,
  }}>
    <div style={{
      fontSize: 11, letterSpacing: 3, color: dsSpecColors.cyan,
      textTransform: "uppercase", marginBottom: 8,
    }}>
      [spec] sphragis · desktop chrome + shell pane · v0.5.0-dev
    </div>
    <div style={{ fontSize: 24, letterSpacing: 2, marginBottom: 4 }}>
      Implementation reference
    </div>
    <div style={{ fontSize: 12, color: dsSpecColors.dim, marginBottom: 32 }}>
      Native target: 1280×800 BGRA8. Title bar 24px · Content 748px · Status bar 28px.
    </div>

    {/* — VERTICAL LAYOUT — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: dsSpecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>vertical layout</div>
    <div style={{
      display: "flex", border: `1px solid ${dsSpecColors.hair}`, marginBottom: 32,
      fontSize: 11,
    }}>
      <div style={{ width: 80, padding: 12, color: dsSpecColors.dim, borderRight: `1px solid ${dsSpecColors.hair}`, textAlign: "center" }}>y=0..24</div>
      <div style={{ flex: 1, padding: 12 }}>title bar — 24px · bg #0A0A0A · 1px #1A1A1A bottom border</div>
    </div>
    <div style={{ display: "flex", border: `1px solid ${dsSpecColors.hair}`, marginBottom: 8, fontSize: 11 }}>
      <div style={{ width: 80, padding: 12, color: dsSpecColors.dim, borderRight: `1px solid ${dsSpecColors.hair}`, textAlign: "center" }}>y=24..772</div>
      <div style={{ flex: 1, padding: 12 }}>content area — 748px · 1px hairlines top + bottom · apps own internal layout</div>
    </div>
    <div style={{ display: "flex", border: `1px solid ${dsSpecColors.hair}`, marginBottom: 32, fontSize: 11 }}>
      <div style={{ width: 80, padding: 12, color: dsSpecColors.dim, borderRight: `1px solid ${dsSpecColors.hair}`, textAlign: "center" }}>y=772..800</div>
      <div style={{ flex: 1, padding: 12 }}>status bar — 28px · bg #0A0A0A · 1px #1A1A1A top border</div>
    </div>

    {/* — TITLE BAR — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: dsSpecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>title bar (24px tall)</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 32 }}>
      <DSBlock title="Brand block (left)">
        <DSRow k="Width" v="132px (fixed)" />
        <DSRow k="Padding" v="14px L/R · 0 T/B" />
        <DSRow k="Project glyph" v="18×12 · #22D3EE · simplified mark" />
        <DSRow k="Wordmark" v="12px / 700 / tracking 2px · 'SPHRAGIS'" />
        <DSRow k="Underscore" v="cyan #22D3EE (only the '_')" />
        <DSRow k="Right border" v="1px #1A1A1A" />
      </DSBlock>
      <DSBlock title="Cave block (right)">
        <DSRow k="Width" v="168px min, right-aligned" />
        <DSRow k="Padding" v="14px L/R" />
        <DSRow k="Label 'CAVE'" v="10px / tracking 1.5px / dim #4B5563" />
        <DSRow k="Cave name" v="11px / ink #E5E7EB" />
        <DSRow k="Status dot" v="6px · green/amber/red · 1px outer ring" />
        <DSRow k="States" v="green=ok · amber=warn · red=wipe-armed" />
      </DSBlock>
    </div>
    <div style={{ marginBottom: 32 }}>
      <DSBlock title="Tab strip (center, 9 tabs · 64px each · 576px total)">
        <DSRow k="Width per tab" v="64px · vertical separator 1px #1A1A1A between tabs" />
        <DSRow k="Digit hint" v="8px · '⌃N' · faint #374151 inactive · cyan #22D3EE active" />
        <DSRow k="Letter pair" v="11px / tracking 1.5px · 700 active · 500 inactive" />
        <DSRow k="Active text" v="#E5E7EB ink" />
        <DSRow k="Inactive text" v="#4B5563 dim" />
        <DSRow k="Active underline" v="2px tall · cyan #22D3EE · 6px inset L/R · sits flush at y=22" />
        <DSRow k="Hover" v="optional — text lifts dim → mid #9CA3AF · NO background fill" />
        <DSRow k="Order" v="SH · DS · FS · NM · ED · SK · CM · WB · BC" />
        <DSRow k="Bind" v="Ctrl+1..9 maps left-to-right" />
      </DSBlock>
    </div>

    {/* — STATUS BAR — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: dsSpecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>status bar (28px tall)</div>
    <div style={{ marginBottom: 16 }}>
      <DSBlock title="Segments — left to right · separated by 1px #262626 vertical">
        <DSRow k="Padding" v="12px L/R per segment · 8px gap between dot/label/value" />
        <DSRow k="Label" v="10px / tracking 1.5px / mid #9CA3AF · uppercase" />
        <DSRow k="Value" v="11px / tracking 1px · color depends on segment" />
        <DSRow k="Seg 1 — ENCRYPTED" v="green dot · no value · ~108px (decoration; always green if booted)" />
        <DSRow k="Seg 2 — NET" v="value e.g. '10.0.2.15' (ink) or 'OFFLINE' (red) · ~150px" />
        <DSRow k="Seg 3 — TLS" v="LOCKDOWN cyan · RESEARCH amber · OPEN red · ~130px" />
        <DSRow k="Seg 4 — JS" v="ON amber · OFF ink · ~70px" />
        <DSRow k="Seg 5 — AUDIT" v="value 'N / MAX' · ink · ~140px" />
        <DSRow k="Spacer" v="flex grow · empty" />
        <DSRow k="Right — UPTIME" v="value '0d 00:14:32' · ink · 1Hz colon blink (replace with space)" />
      </DSBlock>
    </div>
    <div style={{ marginBottom: 32 }}>
      <DSBlock title="Status segments — live state vs decoration">
        <DSRow k="ENCRYPTED" v="decoration (always green post-boot — kill switch separate)" />
        <DSRow k="NET" v="LIVE — netd publishes ip + offline flag" />
        <DSRow k="TLS" v="LIVE — read from tls-mode setting" />
        <DSRow k="JS" v="LIVE — read from js-mode setting" />
        <DSRow k="AUDIT" v="LIVE — read auditd ring counter" />
        <DSRow k="UPTIME" v="LIVE — clock_monotonic delta from boot" />
      </DSBlock>
    </div>

    {/* — SHELL PANE — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: dsSpecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>shell pane (SH tab)</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <DSBlock title="Layout">
        <DSRow k="Padding" v="16px L/R · 8px T/B (inside hairlines)" />
        <DSRow k="Line height" v="16px (matches 8×16 bitmap font) · 13px in mocks" />
        <DSRow k="Visible lines" v="≈45 with 8px insets · designed for 18-20 'recent' before scroll" />
        <DSRow k="Scrollback overflow" v="HARD top edge · no fade · clip at content top" />
        <DSRow k="Prompt position" v="anchored to bottom of pane · 16px above status bar" />
        <DSRow k="Wrap" v="hard wrap at column · indent continuation 2 spaces" />
      </DSBlock>
      <DSBlock title="Prompt typography">
        <DSRow k="'sphragis'" v="#E5E7EB ink · regular weight" />
        <DSRow k="' > '" v="cyan #22D3EE · 6px L/R padding" />
        <DSRow k="Typed text" v="#E5E7EB ink" />
        <DSRow k="Cursor" v="8×14 solid block · cyan #22D3EE · 1Hz blink" />
        <DSRow k="Echo prefix (history)" v="'sphragis' #4B5563 dim · '>' #0E7490 cyanDim — visually receded" />
      </DSBlock>
    </div>
    <div style={{ marginBottom: 32 }}>
      <DSBlock title="Output color rules per category">
        <DSRow k="echo (cmd echo)" v="prefix #4B5563 dim · cmd text #E5E7EB ink" />
        <DSRow k="out (normal)" v="#E5E7EB ink — 2-space indent under cmd" />
        <DSRow k="audit index '[N]'" v="#22D3EE cyan" />
        <DSRow k="audit category 'fetch:' etc" v="#9CA3AF mid" />
        <DSRow k="ok / success markers" v="#22C55E green" />
        <DSRow k="warn (warning)" v="#F59E0B amber · prefix 'warn:' included" />
        <DSRow k="err / FAIL" v="#EF4444 red · prefix 'err:' included" />
        <DSRow k="banner / hints" v="#9CA3AF mid · #4B5563 dim · cyan call-outs for keywords" />
      </DSBlock>
    </div>

    {/* — GLYPH STRATEGY — */}
    <div style={{ marginBottom: 32 }}>
      <DSBlock title="Project glyph variants — same DNA, two rasters">
        <DSRow k="Lock screen" v="120×72 detailed · membranes + finger bones + ears + eye slits + circuit nodes" />
        <DSRow k="Title bar" v="18×12 simplified · membranes + ears only · NO finger bones (collapse to noise)" />
        <DSRow k="Shell banner" v="36×24 detailed · same as lock-screen, just smaller raster tile" />
        <DSRow k="Recommendation" v="Two prebaked tiles — 18×12 and 120×72. Banner downsamples the 120 tile to 36 px." />
        <DSRow k="Tinting" v="Single-color polygon fill — swap fill color in your paint stack, no separate raster per state." />
      </DSBlock>
    </div>

    {/* — KEY BINDINGS — */}
    <DSBlock title="Key bindings (display only — not state)">
      <DSRow k="Ctrl+1..9" v="switch tab SH · DS · FS · NM · ED · SK · CM · WB · BC" />
      <DSRow k="Tab" v="cycle tab forward (display hint in banner)" />
      <DSRow k="Enter" v="submit shell command" />
      <DSRow k="↑ / ↓" v="history scrollback (no autocomplete)" />
      <DSRow k="Esc" v="cancel input · clear current line" />
    </DSBlock>
  </div>
);

window.ShellSpecs = ShellSpecs;
