// Title-bar bat glyph — simplified sibling of the lock-screen mark.
// 18x12 default. Membrane silhouette only (no finger bones, no eye slits —
// those collapse to noise at this size). Same wing-shape DNA.

const BatMini = ({ size = 18, color = "#22D3EE" }) => {
  return (
    <svg
      width={size}
      height={(size * 12) / 18}
      viewBox="0 0 36 24"
      fill="none"
      shapeRendering="crispEdges"
      style={{ display: "block" }}
    >
      {/* left wing */}
      <polygon
        fill={color}
        points="
          18,8
          16,6
          12,4
          6,3
          1,5
          0,9
          3,10
          1,12
          5,13
          3,16
          8,16
          6,19
          11,17
          10,20
          15,18
          16,20
          17,17
        "
      />
      {/* right wing (mirror) */}
      <polygon
        fill={color}
        points="
          18,8
          20,6
          24,4
          30,3
          35,5
          36,9
          33,10
          35,12
          31,13
          33,16
          28,16
          30,19
          25,17
          26,20
          21,18
          20,20
          19,17
        "
      />
      {/* head + ears */}
      <polygon
        fill={color}
        points="
          16,6
          16,2
          17,4
          18,1
          19,4
          20,2
          20,6
        "
      />
      {/* torso */}
      <polygon
        fill={color}
        points="
          16,6
          20,6
          19,14
          18,16
          17,14
        "
      />
    </svg>
  );
};

window.BatMini = BatMini;
