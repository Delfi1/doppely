import type { HudUpdated } from "../bevy";
import { font, hint, panel, row, theme } from "../theme";

export function Hud({ hud }: { hud: HudUpdated | null }) {
  const fps = hud?.fps ?? 0;
  const pos = hud?.position;
  const fpsColor = fps >= 30 ? theme.muted : theme.danger;

  return (
    <node
      style={{
        positionType: "absolute",
        left: 0,
        top: 0,
        width: "100%",
        height: "100%",
        padding: 16,
        flexDirection: "row",
        justifyContent: "spaceBetween",
        alignItems: "flexStart",
        focusPolicy: "pass",
      }}
    >
      <node style={panel({ padding: 10, borderRadius: 10, gap: 4, focusPolicy: "pass" })}>
        <node style={{ ...row, gap: 6 }}>
          <node
            style={{
              width: 6,
              height: 6,
              borderRadius: 999,
              backgroundColor: theme.ok,
            }}
          />
          <text style={{ color: theme.text, fontSize: 12, fontFamily: font.mono }}>
            {hud?.server ?? "—"}
          </text>
        </node>

        <node style={{ ...row, gap: 10 }}>
          <text style={{ ...hint, color: fpsColor, fontFamily: font.mono }}>
            {Math.round(fps)} fps
          </text>
          <text style={{ ...hint, fontFamily: font.mono }}>
            {pos ? pos.map((v) => v.toFixed(1)).join(" ") : "—"}
          </text>
        </node>
      </node>

      <text style={hint}>Esc — пауза</text>
    </node>
  );
}
