// Three Sphragis apps: DS · NM · SK
// Each is a 1280×748 pane intended to sit inside the desktop chrome.

// — DS · DASHBOARD —————————————————————————————————————————

const DashboardPane = ({ narrow = false }) => {
  const cols = narrow ? "1fr" : "1fr 1fr";
  return (
    <div style={{
      width: "100%", height: "100%", background: appColors.bg,
      padding: 16, display: "flex", flexDirection: "column", gap: 16,
      fontFamily: appMono, overflow: "hidden",
    }}>
      <div style={{ display: "grid", gridTemplateColumns: cols, gap: 16, height: 360 }}>
        {/* SYSTEM */}
        <Panel title="SYSTEM" metric="ARM64 / APPLE-M4">
          <TileGrid tiles={[
            { label: "UPTIME",  value: "0d 14m",   unit: "since boot" },
            { label: "FREE MEM", value: "31.2",    unit: "MiB · 4 GiB total" },
            { label: "AUDIT",   value: "247", unit: "/ 1024 ring · ", dot: "neutral" },
            { label: "NETWORK", value: "ONLINE",   unit: "10.0.2.15", dot: "ok", state: "ok" },
          ]} />
        </Panel>

        {/* SECURITY */}
        <Panel title="SECURITY" metric="7 SUBSYSTEMS">
          <KV label="ENCRYPT"  value="AES-256-CTR"   state="neutral" labelW={96} />
          <KV label="HASH"     value="SHA-256"       state="neutral" labelW={96} />
          <KV label="KDF"      value="16 ROUNDS · pre-Argon2id" state="plan" labelW={96} />
          <KV label="FIREWALL" value="DENY ALL"      state="ok" dot="ok" labelW={96} />
          <KV label="AUTH"     value="PASSPHRASE"    state="neutral" labelW={96} />
          <KV label="CAPS"     value="ENFORCED"      state="ok" labelW={96} />
          <KV label="AUDIT"    value="247 / 1024"    state="neutral" labelW={96} />
        </Panel>
      </div>

      {/* ARCHITECTURE — full width */}
      <Panel title="ARCHITECTURE" metric="/etc/release">
        <div style={{
          display: "flex", justifyContent: "space-between", alignItems: "center",
          flex: 1,
        }}>
          <div style={{ fontSize: 13, lineHeight: "20px", whiteSpace: "pre" }}>
            <div style={{ color: appColors.ink, letterSpacing: 1 }}>
              Sphragis  <span style={{ color: appColors.cyan }}>v0.5.0-DEV</span>
            </div>
            <div style={{ color: appColors.mid }}>
              Bare-metal AArch64 microkernel · zero external deps
            </div>
            <div style={{ color: appColors.mid }}>
              Cave isolation · SealFS encrypted · audit-everything
            </div>
            <div style={{ color: appColors.dim, marginTop: 4 }}>
              Built 20260502.a3f1c · signed
            </div>
            <div style={{ color: appColors.faint, marginTop: 8, fontSize: 11 }}>
              compiled with rustc 1.81-nightly · target aarch64-unknown-none-softfloat
            </div>
          </div>
          {!narrow && (
            <div style={{ paddingRight: 8 }}>
              <BatGlyph size={72} stroke={appColors.cyan} node={appColors.cyan} dim={appColors.cyanDim} />
            </div>
          )}
        </div>
      </Panel>
    </div>
  );
};

// — NM · NETMON ————————————————————————————————————————————

const NetMonPane = ({ narrow = false }) => {
  const cols = narrow ? "1fr" : "1fr 1fr";
  return (
    <div style={{
      width: "100%", height: "100%", background: appColors.bg,
      padding: 16, display: "flex", flexDirection: "column", gap: 16,
      fontFamily: appMono, overflow: "hidden",
    }}>
      <div style={{ display: "grid", gridTemplateColumns: cols, gap: 16, height: 280 }}>
        {/* INTERFACE */}
        <Panel
          title="INTERFACE"
          metric={
            <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
              LINK <span style={{
                width: 6, height: 6, background: appColors.green,
                boxShadow: `0 0 0 1px ${appColors.greenDim}`,
                display: "inline-block",
              }} />
            </span>
          }
        >
          <KV label="LINK" value="UP · 1 Gbps full-duplex" state="ok" dot="ok" labelW={56} />
          <KV label="MAC"  value="52:54:00:12:34:56" state="neutral" labelW={56} />
          <KV label="IPv4" value="10.0.2.15 / 24" state="neutral" labelW={56} />
          <KV label="GW"   value="10.0.2.2" state="neutral" labelW={56} />
          <KV label="DNS"  value="10.0.2.3 (DoH)" state="neutral" labelW={56} />
          <KV label="MTU"  value="1500" state="plan" labelW={56} />
        </Panel>

        {/* FIREWALL */}
        <Panel
          title="FIREWALL"
          metric={
            <span style={{
              display: "inline-flex", alignItems: "center", gap: 6,
              padding: "2px 6px",
              border: `1px solid ${appColors.red}`, color: appColors.red,
              fontSize: 9, letterSpacing: 1.5,
            }}>DENY ALL</span>
          }
        >
          <KV label="POLICY"   value="DENY ALL · default-drop" state="fail" labelW={88} />
          <KV label="MODE"     value="ALLOWLIST" state="neutral" labelW={88} />
          <KV label="ALLOWED"  value="8 421" state="ok" labelW={88} />
          <KV label="BLOCKED"  value="142" state="fail" labelW={88} />
          <KV label="LAST EVT" value="OUT 443 → cdn.example.com" state="neutral" labelW={88} />
          <KV label="LAST DROP" value="IN 22 ← 10.0.2.99 (no rule)" state="fail" labelW={88} />
        </Panel>
      </div>

      {/* SECURITY STACK — full width flow */}
      <Panel title="SECURITY STACK" metric="REQUEST FLOW · LIVE">
        {narrow ? (
          <div style={{
            display: "grid", gridTemplateColumns: "repeat(3, 1fr)",
            gap: 16, padding: 12, flex: 1,
          }}>
            <FlowBox label="APP"      sub="sphragis shell"        state="ok" />
            <FlowBox label="TLS 1.3"  sub="LOCKDOWN"            state="ok" />
            <FlowBox label="PIN VRFY" sub="3 PINS · 0 MISMATCH" state="ok" />
            <FlowBox label="SOP"      sub="origin allowlist"    state="ok" />
            <FlowBox label="FIREWALL" sub="DENY ALL"            state="ok" />
            <FlowBox label="WIRE"     sub="virtio-net"          state="ok" />
          </div>
        ) : (
          <Flow steps={[
            { label: "APP",      sub: "sphragis shell",         state: "ok" },
            { label: "TLS 1.3",  sub: "LOCKDOWN",             state: "ok" },
            { label: "PIN VRFY", sub: "3 PINS · 0 MISMATCH",  state: "ok" },
            { label: "SOP",      sub: "origin allowlist",     state: "ok" },
            { label: "FIREWALL", sub: "DENY ALL",             state: "ok" },
            { label: "WIRE",     sub: "virtio-net",           state: "ok" },
          ]} />
        )}
      </Panel>
    </div>
  );
};

// — SK · SECURITY ——————————————————————————————————————————

const SecurityPane = ({ narrow = false }) => {
  return (
    <div style={{
      width: "100%", height: "100%", background: appColors.bg,
      padding: 16, display: "flex", flexDirection: "column", gap: 16,
      fontFamily: appMono, overflow: "hidden",
    }}>
      {/* CAVES — top full width */}
      <Panel title="ACTIVE CAVES" metric="3 / 32 SLOTS">
        <div style={{ display: "flex", flexDirection: "column" }}>
          {/* header row */}
          <div style={{
            display: "flex", alignItems: "center", gap: 12,
            height: 18, padding: "0 4px",
            fontSize: 9, letterSpacing: 1.5, color: appColors.faint,
            textTransform: "uppercase",
            borderBottom: `1px solid ${appColors.hair}`,
          }}>
            <div style={{ width: 32 }}>STATE</div>
            <div style={{ flex: 1 }}>NAME</div>
            <div>CAPABILITIES</div>
          </div>
          <CaveRow state="ok"   badge="RUN" name="kernel"      caps={["NET","DSP","FS"]} />
          <CaveRow state="ok"   badge="RUN" name="research-01" caps={["NET","RAW"]} />
          <CaveRow state="plan" badge="STP" name="sandbox"     caps={["FS"]} />
          <CaveEmptyRow />
        </div>
      </Panel>

      <div style={{
        display: "grid", gridTemplateColumns: narrow ? "1fr" : "1fr 1fr",
        gap: 16, flex: 1,
      }}>
        {/* PIPELINE */}
        <Panel title="SECURITY PIPELINE" metric="8 STAGES">
          <KV label="FIREWALL" value="ACTIVE"               state="ok" dot="ok" labelW={88} />
          <KV label="AES-256"  value="ACTIVE"               state="ok" dot="ok" labelW={88} />
          <KV label="TLS 1.3"  value="LOCKDOWN · 1 PIN"     state="neutral" dot="neutral" labelW={88} />
          <KV label="VPN"      value="STANDBY"              state="plan" dot="plan" labelW={88} />
          <KV label="Tor"      value="3-HOP CIRCUIT"        state="ok" dot="ok" labelW={88} />
          <KV label="DNS"      value="DoH ENABLED"          state="ok" dot="ok" labelW={88} />
          <KV label="AUDIT"    value="247 / 1024 ENTRIES"   state="neutral" dot="neutral" labelW={88} />
          <KV label="DMS"      value="ARMED · 48H"          state="ok" dot="ok" labelW={88} />
        </Panel>

        {/* INTEGRITY */}
        <Panel title="INTEGRITY" metric="SealFS · MERKLE">
          {/* Merkle row */}
          <div style={{
            display: "flex", alignItems: "center", gap: 12,
            height: 22, fontSize: 12,
            paddingBottom: 4,
            borderBottom: `1px solid ${appColors.hair}`,
          }}>
            <div style={{
              width: 88, color: appColors.mid, letterSpacing: 1.5,
              textTransform: "uppercase", fontSize: 10,
            }}>MERKLE</div>
            <span style={{
              color: appColors.ink, fontFamily: appMono,
              letterSpacing: 1, fontVariantNumeric: "tabular-nums",
            }}>c4e3 d7a2 b1f0 8e95</span>
            <span style={{ color: appColors.dim }}>…</span>
            <span style={{
              marginLeft: "auto", color: appColors.green, fontSize: 10, letterSpacing: 1.5,
            }}>VERIFIED</span>
          </div>

          <div style={{ marginTop: 6 }}>
            <KV label="AUTH"       value="VERIFIED"            state="ok"   dot="ok" labelW={88} />
            <KV label="OPEN PORTS" value="0 · invisible-by-design" state="ok" dot="ok" labelW={88} />
            <KV label="WIPE"       value="ARMED"               state="warn" dot="warn" labelW={88} />
          </div>

          <div style={{ flex: 1 }} />

          <div style={{
            marginTop: 8, paddingTop: 8,
            borderTop: `1px solid ${appColors.hair}`,
          }}>
            <AuditStrip lines={[
              { idx: 243, cat: "script:", text: "exec js 1024B" },
              { idx: 242, cat: "fetch :", text: "GET http://10.0.2.2:8765/  OK" },
              { idx: 241, cat: "nav   :", text: "main origin -> http://10.0.2.2" },
              { idx: 240, cat: "mode  :", text: "js-mode -> on" },
            ]} />
          </div>
        </Panel>
      </div>
    </div>
  );
};

Object.assign(window, { DashboardPane, NetMonPane, SecurityPane });
