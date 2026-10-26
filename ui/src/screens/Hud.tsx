import { bevy, type HudUpdated } from "../bevy";
import { column, label, panel, row, theme, transition } from "../theme";

/**
 * Игровой HUD: состояние связи, FPS, координаты и кнопка паузы. Режим паузы
 * не рисует HUD вовсе — его заменяет Esc-меню.
 */
export function Hud({ hud }: { hud: HudUpdated | null }) {
  const fps = hud?.fps ?? 0;
  const pos = hud?.position;

  return (
    <node
      style={{
        positionType: "absolute",
        width: "100%",
        height: "100%",
        padding: 16,
        flexDirection: "column",
        justifyContent: "spaceBetween",
        focusPolicy: "pass",
      }}
    >
      <node style={{ ...row, justifyContent: "spaceBetween", alignItems: "flexStart" }}>
        <node style={panel({ padding: 12, borderRadius: 10, focusPolicy: "pass" })}>
          <node style={column(4)}>
            <node style={{ ...row, gap: 8 }}>
              <node
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: 999,
                  backgroundColor: theme.ok,
                }}
              />
              <text style={{ color: theme.text, fontSize: 13, fontWeight: "semibold" }}>
                Подключено
              </text>
            </node>
            <text style={{ color: theme.faint, fontSize: 11 }}>{hud?.server ?? "—"}</text>
          </node>
        </node>

        <node style={{ ...row, gap: 10 }}>
          <node style={panel({ padding: 12, borderRadius: 10, focusPolicy: "pass" })}>
            <node style={column(4)}>
              <text style={label}>FPS</text>
              <text
                style={{
                  color: fps >= 55 ? theme.ok : fps >= 30 ? theme.warn : theme.danger,
                  fontSize: 18,
                  fontWeight: "bold",
                }}
              >
                {String(fps)}
              </text>
            </node>
          </node>

          <button
            onClick={() => bevy.pause({})}
            style={{
              ...row,
              justifyContent: "center",
              padding: { horizontal: 12, vertical: 10 },
              borderRadius: 10,
              border: 1,
              borderColor: theme.border,
              backgroundColor: theme.surface,
              cursor: "pointer",
              transition,
            }}
            hoverStyle={{ borderColor: theme.accent }}
          >
            <text style={{ color: theme.text, fontSize: 13 }}>Esc · пауза</text>
          </button>
        </node>
      </node>

      <node style={{ ...row, justifyContent: "spaceBetween", alignItems: "flexEnd" }}>
        <node style={panel({ padding: 12, borderRadius: 10, focusPolicy: "pass" })}>
          <node style={column(4)}>
            <text style={label}>ПОЗИЦИЯ</text>
            <text style={{ color: theme.text, fontSize: 12 }}>
              {pos
                ? `${pos[0].toFixed(1)}  ${pos[1].toFixed(1)}  ${pos[2].toFixed(1)}`
                : "ожидание сервера…"}
            </text>
          </node>
        </node>

        <node style={panel({ padding: 12, borderRadius: 10, focusPolicy: "pass" })}>
          <node style={{ ...row, gap: 6 }}>
            <text style={label}>ТИК</text>
            <text style={{ color: theme.text, fontSize: 12 }}>{String(hud?.tick ?? 0)}</text>
          </node>
        </node>
      </node>
    </node>
  );
}
