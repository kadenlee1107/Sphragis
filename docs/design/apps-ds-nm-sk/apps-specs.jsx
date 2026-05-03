// Spec sheet for the DS / NM / SK app trio.

const aspecColors = {
  bg: "#0A0A0A", panel: "#0E0E0E",
  hair: "#1A1A1A", hairHi: "#262626",
  ink: "#E5E7EB", mid: "#9CA3AF", dim: "#4B5563", faint: "#374151",
  cyan: "#22D3EE", green: "#22C55E", amber: "#F59E0B", red: "#EF4444",
};
const aspecMono = `"JetBrains Mono", "IBM Plex Mono", "SF Mono", Menlo, monospace`;

const ARow = ({ k, v }) => (
  <div style={{
    display: "flex", justifyContent: "space-between", gap: 16,
    padding: "6px 0",
    borderBottom: `1px solid ${aspecColors.hair}`,
    fontSize: 11, letterSpacing: 1,
  }}>
    <span style={{ color: aspecColors.dim, textTransform: "uppercase", flexShrink: 0 }}>{k}</span>
    <span style={{ color: aspecColors.ink, textAlign: "right" }}>{v}</span>
  </div>
);

const ABlock = ({ title, children }) => (
  <div style={{
    border: `1px solid ${aspecColors.hair}`, background: "#0E0E0E",
    padding: 16,
  }}>
    <div style={{
      fontSize: 10, letterSpacing: 2, color: aspecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>{title}</div>
    {children}
  </div>
);

const AppsSpecs = () => (
  <div style={{
    width: 1280, padding: 56, background: aspecColors.bg,
    color: aspecColors.ink, fontFamily: aspecMono,
  }}>
    <div style={{
      fontSize: 11, letterSpacing: 3, color: aspecColors.cyan,
      textTransform: "uppercase", marginBottom: 8,
    }}>
      [spec] bat_os · ds · nm · sk · v0.5.0-dev
    </div>
    <div style={{ fontSize: 24, letterSpacing: 2, marginBottom: 4 }}>
      Implementation reference
    </div>
    <div style={{ fontSize: 12, color: aspecColors.dim, marginBottom: 32 }}>
      Pane size 1280×748 native. Inner padding 16px (gutter between panels also 16px).
      Narrow split-pane variant: 512×748 — collapses 2-col grids to 1-col.
    </div>

    {/* — SHARED PANEL SYSTEM — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: aspecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>shared panel system</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 32 }}>
      <ABlock title="Panel chrome">
        <ARow k="Border" v="1px #262626 (hairline+)" />
        <ARow k="Background" v="#0A0A0A bg (panel sits on bg, not on a fill)" />
        <ARow k="Title strip" v="24px tall · 1px #1A1A1A bottom separator" />
        <ARow k="Title label" v="10px / tracking 2px / #374151 faint · upper" />
        <ARow k="Title metric (right)" v="10px / tracking 1.5px / #4B5563 dim" />
        <ARow k="Padding (title)" v="12px L/R · 0 T/B (centered in 24px)" />
        <ARow k="Body padding" v="16px L/R · 12px T/B" />
        <ARow k="No rounded corners" v="anywhere · square only" />
      </ABlock>
      <ABlock title="Status dot palette mapping">
        <ARow k="ok"      v="green #22C55E · ring #14532D" />
        <ARow k="warn"    v="amber #F59E0B · ring #78350F" />
        <ARow k="fail"    v="red   #EF4444 · ring #7F1D1D" />
        <ARow k="plan"    v="dim   #4B5563 · ring #374151 (idle / not wired)" />
        <ARow k="neutral" v="cyan  #22D3EE · ring #0E7490 (live values: IPs, modes, hashes)" />
        <ARow k="Dot geom" v="6px solid square · 1px outer ring · 8px gap to value" />
      </ABlock>
    </div>

    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 16, marginBottom: 32 }}>
      <ABlock title="TYPE A · KV row">
        <ARow k="Row height" v="22px (16px line · 6px breathing)" />
        <ARow k="Label" v="10px / tracking 1.5 / mid #9CA3AF · upper" />
        <ARow k="Value" v="12px / tabular-nums / colored per state" />
        <ARow k="Label col" v="auto-aligned per panel · see per-panel widths" />
        <ARow k="Gap" v="12px label→value · 8px dot→value" />
      </ABlock>
      <ABlock title="TYPE B · Tile">
        <ARow k="Tile size" v="≥ 80px tall · 1px #1A1A1A grid lines" />
        <ARow k="Top label" v="10px / tracking 2 / #374151 faint" />
        <ARow k="Number" v="24px / tabular-nums / #E5E7EB ink" />
        <ARow k="Unit" v="10px / tracking 2 / #4B5563 dim · 4px below number" />
        <ARow k="Optional dot" v="6px · sits left of top label" />
      </ABlock>
      <ABlock title="TYPE C · Caves row">
        <ARow k="Row height" v="28px · 1px #1A1A1A bottom border" />
        <ARow k="State badge" v="32×16 · 1px outline + 9px label · color = state" />
        <ARow k="Name" v="12px ink #E5E7EB · flex: 1" />
        <ARow k="Cap pill" v="38×16 · cyan #22D3EE on / faint #374151 off" />
        <ARow k="Pill gap" v="4px between pills" />
        <ARow k="Header row" v="18px · 9px / tracking 1.5 / faint · upper" />
      </ABlock>
    </div>

    {/* — DS — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: aspecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>ds · dashboard</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <ABlock title="Layout">
        <ARow k="Top row" v="2 panels × 624×360 · 16px gutter" />
        <ARow k="Bottom row" v="1 panel × 1248×340 (full width)" />
        <ARow k="Narrow (512w)" v="all panels stacked single column" />
        <ARow k="SYSTEM panel" v="2×2 tiles · ARM64 / APPLE-M4 in title metric" />
        <ARow k="SECURITY panel" v="7 KV rows · label col 96px" />
        <ARow k="ARCHITECTURE" v="left-aligned ASCII text · 36×24 bat at right edge" />
      </ABlock>
      <ABlock title="Tile content">
        <ARow k="UPTIME"  v="value '0d 14m' · unit 'since boot' · no dot" />
        <ARow k="FREE MEM" v="value '31.2' · unit 'MiB · 4 GiB total' · no dot" />
        <ARow k="AUDIT" v="value '247' · unit '/ 1024 ring' · cyan dot" />
        <ARow k="NETWORK" v="value 'ONLINE' (green) · unit '10.0.2.15' · green dot" />
      </ABlock>
    </div>

    {/* — NM — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: aspecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>nm · netmon</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <ABlock title="Layout">
        <ARow k="Top row" v="INTERFACE 624×280 · FIREWALL 624×280" />
        <ARow k="Bottom row" v="SECURITY STACK · 1248×420 (flow diagram)" />
        <ARow k="INTERFACE label col" v="56px (LINK / MAC / IPv4 / GW / DNS / MTU)" />
        <ARow k="FIREWALL label col" v="88px (longest: 'LAST DROP')" />
        <ARow k="DENY ALL badge" v="red border · red text · 9px / tracking 1.5 · 2×6 padding" />
      </ABlock>
      <ABlock title="Pipeline flow boxes (SECURITY STACK)">
        <ARow k="Box size" v="110×44 · 1px border in state color · #0E0E0E fill" />
        <ARow k="Box label" v="11px / 700 / tracking 2 · color = state" />
        <ARow k="Sub caption" v="9px / tracking 1.5 / dim · upper · ≤110px wide" />
        <ARow k="Arrow" v="32×44 spacer · 1px line cyanDim #0E7490 · 6×8 cyan triangle head" />
        <ARow k="6 steps" v="APP → TLS 1.3 → PIN VRFY → SOP → FIREWALL → WIRE" />
        <ARow k="Narrow (512w)" v="3×2 grid of FlowBox cells · arrows omitted" />
      </ABlock>
    </div>

    {/* — SK — */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: aspecColors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>sk · security</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <ABlock title="Layout">
        <ARow k="Top" v="ACTIVE BATCAVES · 1248×180 (full width)" />
        <ARow k="Bottom L" v="SECURITY PIPELINE · 624×504" />
        <ARow k="Bottom R" v="INTEGRITY · 624×504" />
        <ARow k="PIPELINE label col" v="88px (longest: 'FIREWALL')" />
        <ARow k="INTEGRITY label col" v="88px" />
      </ABlock>
      <ABlock title="Caves table specifics">
        <ARow k="Header height" v="18px · STATE / NAME / CAPABILITIES" />
        <ARow k="Row height" v="28px" />
        <ARow k="Badges in mocks" v="RUN (green) · STP (dim)" />
        <ARow k="Future badges" v="WPE (red wipe-armed) · PND (amber pending)" />
        <ARow k="Cap order" v="NET · RAW · DSP · FS — fixed columns, lit if present" />
        <ARow k="Empty row" v="11px dim · '(no further caves · 29 slots free)'" />
      </ABlock>
    </div>

    {/* — INTEGRITY DETAIL — */}
    <div style={{ marginBottom: 32 }}>
      <ABlock title="INTEGRITY panel structure (top → bottom)">
        <ARow k="1. Merkle row" v="custom KV-like · 88 label · 'c4e3 d7a2 b1f0 8e95…' tabular · right-aligned 'VERIFIED' green" />
        <ARow k="2. KV trio" v="AUTH · OPEN PORTS · WIPE — all with status dots" />
        <ARow k="OPEN PORTS framing" v="value '0 · invisible-by-design' · 0 is real, framing is rhetorical" />
        <ARow k="3. Audit mini-strip" v="matches lock-screen boot log · '[N]' cyan · cat mid · text ink · 11px / 14px line" />
        <ARow k="Vertical" v="merkle 22px · 8px gap · KVs 66px · flex spacer · audit 80px" />
      </ABlock>
    </div>

    {/* — STUFF NOT WIRED — */}
    <ABlock title="Substitutions (do not fake — keep these labels)">
      <ARow k="YubiKey" v="dropped from AUTH row · only 'PASSPHRASE' shown" />
      <ARow k="Argon2id" v="KDF row reads '16 ROUNDS · pre-Argon2id' (state=plan, dim)" />
      <ARow k="VPN" v="STANDBY (state=plan, dim) — stack present, not connected" />
      <ARow k="MTU" v="dim · plan state · not actively negotiated yet" />
      <ARow k="Wall clock" v="UPTIME shown, not UTC — uptime is monotonic since boot" />
      <ARow k="OPEN PORTS = 0" v="real value · 'invisible-by-design' is positive framing" />
    </ABlock>
  </div>
);

window.AppsSpecs = AppsSpecs;
