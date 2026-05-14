// Spec for WB · BC (Wave 4).

const w4Colors = {
  bg: "#0A0A0A", panel: "#0E0E0E",
  hair: "#1A1A1A", hairHi: "#262626",
  ink: "#E5E7EB", mid: "#9CA3AF", dim: "#4B5563", faint: "#374151",
  cyan: "#22D3EE", green: "#22C55E", amber: "#F59E0B", red: "#EF4444",
};
const w4Mono = `"JetBrains Mono", "IBM Plex Mono", "SF Mono", Menlo, monospace`;

const W4Row = ({ k, v }) => (
  <div style={{
    display: "flex", justifyContent: "space-between", gap: 16,
    padding: "6px 0",
    borderBottom: `1px solid ${w4Colors.hair}`,
    fontSize: 11, letterSpacing: 1,
  }}>
    <span style={{ color: w4Colors.dim, textTransform: "uppercase", flexShrink: 0 }}>{k}</span>
    <span style={{ color: w4Colors.ink, textAlign: "right" }}>{v}</span>
  </div>
);

const W4Block = ({ title, children }) => (
  <div style={{
    border: `1px solid ${w4Colors.hair}`, background: "#0E0E0E",
    padding: 16,
  }}>
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w4Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>{title}</div>
    {children}
  </div>
);

const Wave4Specs = () => (
  <div style={{
    width: 1280, padding: 56, background: w4Colors.bg,
    color: w4Colors.ink, fontFamily: w4Mono,
  }}>
    <div style={{
      fontSize: 11, letterSpacing: 3, color: w4Colors.cyan,
      textTransform: "uppercase", marginBottom: 8,
    }}>
      [spec] sphragis · wb · bc · v0.5.0-dev
    </div>
    <div style={{ fontSize: 24, letterSpacing: 2, marginBottom: 4 }}>
      Implementation reference (wave 4)
    </div>
    <div style={{ fontSize: 12, color: w4Colors.dim, marginBottom: 32 }}>
      Pane 1280×748. WB has 4 strips (40 nav · 24 bookmarks · flex page · 24 status). BC has 3 strips (32 header · flex split body · 28 bottom).
    </div>

    {/* WB · NAV STRIP */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w4Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>wb · nav strip (40px)</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <W4Block title="Layout (left to right)">
        <W4Row k="L padding" v="12px" />
        <W4Row k="Back '<' btn"   v="28×28 · 1px #262626 · panel bg · cyan when enabled / faint when disabled" />
        <W4Row k="Forward '>' btn" v="28×28 · same · normally disabled (no forward history)" />
        <W4Row k="Reload 'R' btn"  v="28×28 · cyan idle · amber while loading (no spinner)" />
        <W4Row k="Gap"           v="6px between buttons · 8px before URL bar" />
        <W4Row k="URL bar"       v="flex 1 · h=28 · panel bg · 1px #262626 (hairline+) idle · 1px #22D3EE focused" />
        <W4Row k="Gap"           v="8px after URL bar" />
        <W4Row k="SOP pill"      v="22px tall · 8px L/R padding · 1px hairline+ · panel bg" />
        <W4Row k="JS pill"       v="22px · same chrome · text 'JS ON' amber / 'JS OFF' ink" />
        <W4Row k="Star btn"      v="28×28 · ASCII '*' · cyan if bookmarked / faint if not" />
      </W4Block>
      <W4Block title="URL bar internals">
        <W4Row k="Lock zone"     v="22px wide · 6px right padding · 1px #1A1A1A right divider · 8px gap to URL" />
        <W4Row k="Lock glyphs"   v="HTTPS-pin '[#]' green · RESEARCH '[#]' amber · HTTP '[/]' red · file/empty ' ? ' dim" />
        <W4Row k="URL text"      v="12px ink #E5E7EB · monospace · ellipsis on overflow" />
        <W4Row k="Cursor (focused)" v="8×2 underscore · cyan #22D3EE · 1Hz blink · sits on baseline of typing position" />
        <W4Row k="Padding"       v="10px L/R inside the bar" />
      </W4Block>
    </div>

    {/* WB · BOOKMARKS / STATUS */}
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 32 }}>
      <W4Block title="Bookmarks bar (24px)">
        <W4Row k="Padding" v="8px L/R" />
        <W4Row k="Bottom border" v="1px #1A1A1A" />
        <W4Row k="Chip" v="24px tall · 12px L/R padding · 8px gap (favicon ↔ host)" />
        <W4Row k="Favicon swatch" v="12×12 solid · 1px #1A1A1A outer ring" />
        <W4Row k="Swatch color rule" v="hash(host) % palette[8] — palette = cyan/green/amber/red/cyanDim/greenDim/amberDim/redDim" />
        <W4Row k="Host text" v="11px ink · letter-spacing 0" />
        <W4Row k="Right hint" v="'N / 32 saved' · 9px / faint / tracking 1.5 · 8px R padding" />
        <W4Row k="Persistence" v="NOT WIRED — assumes future static array" />
      </W4Block>
      <W4Block title="Status strip (24px)">
        <W4Row k="Left segment" v="320px fixed · 1px #262626 right border · 12px L/R padding" />
        <W4Row k="Left text" v="10px / tracking 1.5 / upper · color per state (READY=dim · TLS HANDSHAKE=amber · RX=cyan · RENDERED=ink)" />
        <W4Row k="Mid segment" v="flex · 12px L/R padding · empty unless 0&lt;progress&lt;1" />
        <W4Row k="Progress bar" v="100% wide · 1px tall · cyanDim #0E7490 trail · cyan #22D3EE filled portion" />
        <W4Row k="Right segment" v="auto · 1px #262626 left border · 12px L/R padding" />
        <W4Row k="Right content" v="6px dot (green if &gt;0, faint if 0) + tabular count + 'COOKIES' label" />
      </W4Block>
    </div>

    {/* WB · STATES */}
    <div style={{ marginBottom: 32 }}>
      <W4Block title="WB states (3 artboards)">
        <W4Row k="A · Idle" v="URL='' focused (cyan border) · cursor blinking · status 'READY' dim · no progress · 0 cookies" />
        <W4Row k="B · Loading" v="URL='https://example.com/' · lock amber · status 'TLS HANDSHAKE…' amber · progress 60% · reload btn amber" />
        <W4Row k="C · Loaded" v="URL same · lock green · status 'RENDERED 8421B / 47 nodes / 89 boxes' ink · 3 cookies (green dot) · faux page rendered" />
      </W4Block>
    </div>

    {/* BC · LAYOUT */}
    <div style={{
      fontSize: 10, letterSpacing: 2, color: w4Colors.faint,
      textTransform: "uppercase", marginBottom: 10,
    }}>bc · layout</div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <W4Block title="Header (32px) — same as FS">
        <W4Row k="Left" v="'CAVES' 12px / 700 / tracking 2 + dim subtitle 'Isolated container runtime'" />
        <W4Row k="Right" v="'N / 32 SLOTS' · tabular count ink · label dim" />
      </W4Block>
      <W4Block title="Body split">
        <W4Row k="Left (table)" v="60% width · 1px #1A1A1A right divider" />
        <W4Row k="Right (detail)" v="40% width · scrollable interior" />
        <W4Row k="Empty table"  v="centered dim · '(no Caves — use caves create &lt;name&gt; in shell)'" />
      </W4Block>
    </div>

    {/* BC · TABLE */}
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <W4Block title="Table columns (16px L/R margin · 12px col gap)">
        <W4Row k="STATE"        v="60px · 32×16 badge (RUN green / STP dim / WPE amber)" />
        <W4Row k="NAME"         v="flex · 12px ink · letter-spacing 0" />
        <W4Row k="TYPE"         v="60px · 44×16 pill (PERS cyan / EPHM amber)" />
        <W4Row k="CAPABILITIES" v="200px · 4× 38×16 cap pills · 4px gap · NET RAW DSP FS · cyan on / faint off" />
        <W4Row k="Header row"   v="22px · 9px / tracking 1.5 / faint · 1px hair below" />
        <W4Row k="Data row"     v="28px · 1px hair below" />
      </W4Block>
      <W4Block title="Selection (matches FS)">
        <W4Row k="Border"     v="1px #0E7490 cyanDim around row" />
        <W4Row k="Underline"  v="2px #22D3EE strip at row bottom · inset 16px L/R" />
        <W4Row k="Background" v="UNCHANGED (#0A0A0A bg)" />
      </W4Block>
    </div>

    {/* BC · DETAIL */}
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <W4Block title="Detail panel — populated">
        <W4Row k="Padding"    v="16px L/R · 16px top · 12px bottom" />
        <W4Row k="Glyph"      v="64×48 SVG · cyan running · amber wiping · dim stopped · dashed inner seal · corner notches" />
        <W4Row k="Title"      v="16px / 700 · ink · cave name" />
        <W4Row k="Subtitle"   v="10px / tracking 1.5 · '● STATE · TYPE_LONG' · state-colored dot + label" />
        <W4Row k="KV label col" v="72px (longest 'CREATED')" />
        <W4Row k="KV rows"    v="NAME · STATE · TYPE · FS_KEY · CAPS · TOOLS · AUDIT · CREATED" />
        <W4Row k="FS_KEY"     v="first 8 hex of derived key · 'wiped' red when state=warn" />
        <W4Row k="TOOLS=0"    v="kernel cave · displayed '0 (kernel cave)' dim" />
      </W4Block>
      <W4Block title="Action hints + audit strip">
        <W4Row k="Section label" v="'ACTIONS' 10px / tracking 2 / faint · upper" />
        <W4Row k="Line layout" v="'[shell] {cmd}  # {comment}' · 11px / 18px line" />
        <W4Row k="Prefix '[shell]'" v="faint #374151 · letter-spacing 1 · upper" />
        <W4Row k="Cmd"         v="cyan #22D3EE normal · amber #F59E0B for irreversible (seal/destroy)" />
        <W4Row k="Comment"     v="faint #374151 · prefix '#' · right-aligned" />
        <W4Row k="Audit strip" v="bottom of panel · 1px #1A1A1A top · 8px above strip · 4 lines · reuses lock-screen log style" />
        <W4Row k="No buttons"  v="EXPLICITLY no clickable action buttons — keyboard-driven" />
      </W4Block>
    </div>

    {/* BC · DETAIL EMPTY + BOTTOM */}
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
      <W4Block title="Detail panel — empty (quickstart)">
        <W4Row k="Top text" v="centered dim 12px · '(no cave selected)' · 16px below" />
        <W4Row k="Section label" v="'QUICK START' 10px / tracking 2 / faint" />
        <W4Row k="Quickstart line" v="cmd cyan + faint trailing '# comment' · 11px / 18px" />
        <W4Row k="Lines" v="create · grant net · grant raw display · enter · seal · destroy" />
      </W4Block>
      <W4Block title="Bottom strip (28px)">
        <W4Row k="Left seg" v="'CAVES N · MAX 32 · RUNNING M' · 1px #262626 right · 16px L/R" />
        <W4Row k="Mid seg" v="3× pills RUN/STP/DEL · 18px tall · 8px L/R · 9px / tracking 1.5 · color-per-state border" />
        <W4Row k="Right hint" v="'↑↓ select · Enter focus · shell to manage' · 10px dim · arrows + 'Enter' faint" />
      </W4Block>
    </div>

    {/* BC · STATES */}
    <div style={{ marginBottom: 32 }}>
      <W4Block title="BC states (3 artboards)">
        <W4Row k="A · Empty" v="0 caves · table empty-state · detail panel quickstart · bottom counts 0/32/0" />
        <W4Row k="B · Two caves" v="kernel + research-01 · kernel selected · detail populated (PERS, FS_KEY c4e3d7a2, CAPS NET DSP FS, TOOLS 0, AUDIT 247)" />
        <W4Row k="C · Wipe" v="kernel + research-01(WPE) · research-01 selected · STATE WIPE amber · FS_KEY 'wiped' red · audit shows 'destroy initiated'" />
      </W4Block>
    </div>

    {/* SUBSTITUTIONS */}
    <W4Block title="Substitutions (do not fake)">
      <W4Row k="WB · bookmarks" v="not persisted yet — design assumes future static array · placeholder host names only" />
      <W4Row k="WB · favicons" v="hash-derived 12×12 swatch · NO real favicon fetch in scope" />
      <W4Row k="WB · lock state" v="reads tls_pinning::current_mode · not a TLS validator" />
      <W4Row k="WB · JS pill" v="reads existing js-mode toggle (also shown in desktop status bar)" />
      <W4Row k="WB · RX bytes" v="actual fetch byte count from net stack · don't invent" />
      <W4Row k="BC · AUDIT N" v="audit ring filtered by Category::Cave (STUMP #111) · real count" />
      <W4Row k="BC · TOOLS" v="kernel cave is 0 · only docker-backed caves have tool lists" />
      <W4Row k="BC · FS_KEY" v="derived per cave from master key + name · ONLY first 8 hex are safe to render" />
      <W4Row k="BC · no buttons" v="keyboard-only · action hints tell operator what to type — NOT clickable" />
    </W4Block>
  </div>
);

window.Wave4Specs = Wave4Specs;
