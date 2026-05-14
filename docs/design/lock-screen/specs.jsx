// Specs sheet — concrete sizes, hex values, positions for the Rust impl.

const specColors = {
  bg: "#0A0A0A",
  ink: "#E5E7EB",
  dim: "#9CA3AF",
  faint: "#4B5563",
  hair: "#1A1A1A",
  hairHi: "#262626",
  cyan: "#22D3EE",
  green: "#22C55E",
  amber: "#F59E0B",
  red: "#EF4444",
};

const monoSpec = `"JetBrains Mono", "IBM Plex Mono", "SF Mono", Menlo, monospace`;

const Swatch = ({ name, hex, sub }) => (
  <div style={{
    display: "flex", alignItems: "center", gap: 12,
    padding: "10px 12px",
    border: `1px solid ${specColors.hair}`,
    background: "#0E0E0E",
  }}>
    <div style={{
      width: 36, height: 36, background: hex,
      boxShadow: `0 0 0 1px ${specColors.hairHi}`,
    }} />
    <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
      <div style={{ color: specColors.ink, fontSize: 11, letterSpacing: 1.5, textTransform: "uppercase" }}>{name}</div>
      <div style={{ color: specColors.cyan, fontSize: 12 }}>{hex}</div>
      {sub && <div style={{ color: specColors.faint, fontSize: 10, letterSpacing: 1 }}>{sub}</div>}
    </div>
  </div>
);

const Row = ({ k, v }) => (
  <div style={{
    display: "flex", justifyContent: "space-between",
    padding: "6px 0",
    borderBottom: `1px solid ${specColors.hair}`,
    fontSize: 11, letterSpacing: 1,
  }}>
    <span style={{ color: specColors.dim, textTransform: "uppercase" }}>{k}</span>
    <span style={{ color: specColors.ink }}>{v}</span>
  </div>
);

const Block = ({ title, children }) => (
  <div style={{
    border: `1px solid ${specColors.hair}`, background: "#0E0E0E",
    padding: 16,
  }}>
    <div style={{
      fontSize: 10, letterSpacing: 2, color: specColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>{title}</div>
    {children}
  </div>
);

const Specs = () => (
  <div style={{
    width: 1280, padding: 56, background: specColors.bg,
    color: specColors.ink, fontFamily: monoSpec,
  }}>
    <div style={{
      fontSize: 11, letterSpacing: 3, color: specColors.cyan,
      textTransform: "uppercase", marginBottom: 8,
    }}>
      [spec] sphragis · lock screen · v0.4.2-rc1
    </div>
    <div style={{ fontSize: 24, letterSpacing: 2, marginBottom: 4 }}>
      Implementation reference
    </div>
    <div style={{ fontSize: 12, color: specColors.dim, marginBottom: 32 }}>
      Native target: 1280×800 BGRA8. Scales intentionally to 1024×768 (margins compress to 32px, center stack unchanged).
    </div>

    {/* — PALETTE — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: specColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>palette · 32-bit BGRA</div>
    <div style={{
      display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 10,
      marginBottom: 32,
    }}>
      <Swatch name="Background" hex="#0A0A0A" sub="0A 0A 0A FF" />
      <Swatch name="Panel" hex="#0E0E0E" sub="0E 0E 0E FF" />
      <Swatch name="Hairline" hex="#1A1A1A" sub="1A 1A 1A FF" />
      <Swatch name="Hairline+" hex="#262626" sub="26 26 26 FF" />
      <Swatch name="Ink" hex="#E5E7EB" sub="primary text" />
      <Swatch name="Mid" hex="#9CA3AF" sub="labels" />
      <Swatch name="Dim" hex="#4B5563" sub="meta" />
      <Swatch name="Faint" hex="#374151" sub="captions" />
      <Swatch name="Cyan / primary" hex="#22D3EE" sub="status · accent" />
      <Swatch name="Cyan dim" hex="#0E7490" sub="ring · trace" />
      <Swatch name="Green / ok" hex="#22C55E" sub="status dot" />
      <Swatch name="Green dim" hex="#14532D" sub="status ring" />
      <Swatch name="Amber / warn" hex="#F59E0B" sub="attempts dot" />
      <Swatch name="Amber dim" hex="#78350F" sub="attempts ring" />
      <Swatch name="Red / deny" hex="#EF4444" sub="ACCESS DENIED" />
      <Swatch name="Red dim" hex="#7F1D1D" sub="deny ring" />
    </div>

    {/* — TYPE + GEOMETRY — */}
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 32 }}>
      <Block title="Typography · ascii only · monospace only">
        <Row k="Family" v="JetBrains Mono (fallback IBM Plex Mono → SF Mono)" />
        <Row k="Wordmark" v="32px / 700 / tracking 8px" />
        <Row k="Version line" v="11px / 400 / tracking 3px" />
        <Row k="Status pills" v="11px / 500 / tracking 1.2px" />
        <Row k="Field dots" v="8×8 px solid square, 8px gap" />
        <Row k="Cursor" v="10×22 px solid block, 1Hz blink" />
        <Row k="Field label" v="10px / tracking 2px / uppercase" />
        <Row k="Boot log" v="11px / 16px line / tracking 0" />
        <Row k="Clock" v="14px / tracking 2px" />
      </Block>
      <Block title="Geometry · 8px grid">
        <Row k="Canvas" v="1280 × 800 (native)" />
        <Row k="Outer margin" v="56px L/R · 24px T/B" />
        <Row k="Status row" v="y=24 → 52 · 28px tall" />
        <Row k="Hairline" v="y=64 · full inner width · 1px #1A1A1A" />
        <Row k="Project glyph" v="96 × 64 · 2px stroke · centered, y=center−180" />
        <Row k="Wordmark" v="centered, 24px below glyph" />
        <Row k="Field" v="480 × 56 · centered · 1px border" />
        <Row k="Field padding" v="18px L/R · 8px gap between dots" />
        <Row k="Boot log" v="bottom-left · 460px wide · y=h−24−lineH×4" />
        <Row k="Clock block" v="bottom-right · 24px from edges" />
        <Row k="Crosshair marks" v="14px L-shapes at 24px from each corner" />
      </Block>
    </div>

    {/* — STATES — */}
    <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 16, marginBottom: 32 }}>
      <Block title="State A · idle">
        <Row k="Field border" v="#262626 (hairline+)" />
        <Row k="Dots" v="0" />
        <Row k="Cursor" v="#9CA3AF blink" />
        <Row k="Attempts" v="4 / amber dot" />
        <Row k="Glyph" v="#22D3EE" />
      </Block>
      <Block title="State B · typing">
        <Row k="Field border" v="#22D3EE + inset ring #0E7490" />
        <Row k="Dots" v="7 × #E5E7EB" />
        <Row k="Cursor" v="#22D3EE blink" />
        <Row k="Attempts" v="4 / amber dot" />
        <Row k="Glyph" v="#22D3EE" />
      </Block>
      <Block title="State C · denied">
        <Row k="Field border" v="#EF4444" />
        <Row k="Overlay" v="ACCESS DENIED · 28px / 700 / tracking 8 · #EF4444" />
        <Row k="Overlay frame" v="1px #EF4444 + 1px #7F1D1D ring · 28×56 padding" />
        <Row k="Attempts" v="3 / red dot" />
        <Row k="Glyph" v="#EF4444 (re-tint, no redraw)" />
        <Row k="Duration" v="900ms hold → fade to idle" />
      </Block>
    </div>

    {/* — RASTER NOTES — */}
    <Block title="Raster notes for the paint stack">
      <Row k="Gradients" v="None across &gt;5% of screen. Only the 1px scanline overlay (3px repeat, ~1.2% white). Optional: omit if expensive." />
      <Row k="Shadows" v="Crisp 1px outer rings on status dots. No soft blur anywhere." />
      <Row k="Glyph" v="Rasterize once at 96×64 into BGRA tile, blit on draw. Recolor by replacing #22D3EE → #EF4444 in palette LUT for denied state." />
      <Row k="Cursor blink" v="Toggle every 500ms; redraw only the 10×22 cursor rect." />
      <Row k="Failure flash" v="Optional: 1-frame red border (#EF4444) on field at t=0, hold to t=900ms." />
    </Block>
  </div>
);

window.Specs = Specs;
