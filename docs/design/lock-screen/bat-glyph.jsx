// Geometric bat — proper wing anatomy, angular/sharp.
// Symmetric, monochrome, designed for crisp raster.
// viewBox 120x72.

const BatGlyph = ({ size = 120, stroke = "#22D3EE", node = "#22D3EE", dim = "#0E3A44" }) => {
  return (
    <svg
      width={size}
      height={(size * 72) / 120}
      viewBox="0 0 120 72"
      fill="none"
      shapeRendering="crispEdges"
      style={{ display: "block" }}
    >
      {/* ——— LEFT WING ——— */}
      {/* Filled membrane — angular silhouette with scalloped trailing edge */}
      <polygon
        fill={stroke}
        points="
          60,22
          54,18
          44,14
          32,10
          18,8
          6,14
          2,24
          10,28
          4,34
          14,38
          8,46
          22,46
          18,54
          32,50
          30,58
          44,52
          46,58
          56,50
          58,42
        "
      />

      {/* ——— RIGHT WING (mirror) ——— */}
      <polygon
        fill={stroke}
        points="
          60,22
          66,18
          76,14
          88,10
          102,8
          114,14
          118,24
          110,28
          116,34
          106,38
          112,46
          98,46
          102,54
          88,50
          90,58
          76,52
          74,58
          64,50
          62,42
        "
      />

      {/* ——— BODY ——— */}
      {/* Head + ears */}
      <polygon
        fill={stroke}
        points="
          54,18
          54,8
          57,14
          60,4
          63,14
          66,8
          66,18
        "
      />
      {/* Torso wedge */}
      <polygon
        fill={stroke}
        points="
          54,18
          66,18
          64,38
          60,46
          56,38
        "
      />

      {/* ——— FINGER BONES — dim lines on top of membrane for technical feel ——— */}
      {/* left wing */}
      <line x1="56" y1="22" x2="18" y2="8"  stroke={dim} strokeWidth="1" />
      <line x1="56" y1="26" x2="6"  y2="20" stroke={dim} strokeWidth="1" />
      <line x1="56" y1="30" x2="10" y2="32" stroke={dim} strokeWidth="1" />
      <line x1="56" y1="36" x2="14" y2="42" stroke={dim} strokeWidth="1" />
      <line x1="56" y1="42" x2="22" y2="50" stroke={dim} strokeWidth="1" />
      {/* right wing */}
      <line x1="64" y1="22" x2="102" y2="8"  stroke={dim} strokeWidth="1" />
      <line x1="64" y1="26" x2="114" y2="20" stroke={dim} strokeWidth="1" />
      <line x1="64" y1="30" x2="110" y2="32" stroke={dim} strokeWidth="1" />
      <line x1="64" y1="36" x2="106" y2="42" stroke={dim} strokeWidth="1" />
      <line x1="64" y1="42" x2="98"  y2="50" stroke={dim} strokeWidth="1" />

      {/* ——— CIRCUIT NODES at wing tips & joints ——— */}
      <rect x="17" y="7"  width="2" height="2" fill={node} />
      <rect x="5"  y="13" width="2" height="2" fill={node} />
      <rect x="9"  y="27" width="2" height="2" fill={node} />
      <rect x="13" y="37" width="2" height="2" fill={node} />
      <rect x="21" y="45" width="2" height="2" fill={node} />

      <rect x="101" y="7"  width="2" height="2" fill={node} />
      <rect x="113" y="13" width="2" height="2" fill={node} />
      <rect x="109" y="27" width="2" height="2" fill={node} />
      <rect x="105" y="37" width="2" height="2" fill={node} />
      <rect x="97"  y="45" width="2" height="2" fill={node} />

      {/* Eye slits */}
      <rect x="56" y="13" width="2" height="1" fill="#0A0A0A" />
      <rect x="62" y="13" width="2" height="1" fill="#0A0A0A" />

      {/* ——— Subtle circuit traces below body, optional ——— */}
      <line x1="60" y1="46" x2="60" y2="62" stroke={dim} strokeWidth="1" />
      <line x1="52" y1="62" x2="68" y2="62" stroke={dim} strokeWidth="1" />
      <rect x="51" y="61" width="2" height="2" fill={node} />
      <rect x="67" y="61" width="2" height="2" fill={node} />
      <rect x="59" y="61" width="2" height="2" fill={node} />
    </svg>
  );
};

window.BatGlyph = BatGlyph;
