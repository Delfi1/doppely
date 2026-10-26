import { bevy, type ConnectionStatusChanged } from "../bevy";
import { Button } from "../components";
import { column, font, hint, overlay, panel, row, theme } from "../theme";

export function ConnectionScreen({
  status,
  error,
}: {
  status: ConnectionStatusChanged | null;
  error: boolean;
}) {
  return (
    <node style={overlay()}>
      <node style={panel({ width: 320, flexDirection: "column", gap: 12 })}>
        {error ? (
          <>
            <text style={{ color: theme.text, fontSize: 15, fontWeight: "semibold" }}>
              Не удалось подключиться
            </text>
            <text style={{ color: theme.danger, fontSize: 12 }}>
              {status?.message ?? "Соединение потеряно"}
            </text>
            <node style={{ ...row, gap: 6 }}>
              <text style={hint}>Сервер:</text>
              <text style={{ ...hint, color: theme.muted, fontFamily: font.mono }}>
                {status?.server ?? "—"}
              </text>
            </node>
            <node style={column(6)}>
              <Button variant="primary" onClick={() => bevy.leave({})}>
                В главное меню
              </Button>
              <Button variant="danger" onClick={() => bevy.quit({})}>
                Закрыть игру
              </Button>
            </node>
          </>
        ) : (
          <>
            <text style={{ color: theme.text, fontSize: 15, fontWeight: "semibold" }}>
              Подключение
            </text>
            <text style={{ color: theme.muted, fontSize: 12 }}>
              {status?.message ?? "Устанавливаем соединение…"}
            </text>
            <text style={{ ...hint, fontFamily: font.mono }}>{status?.server ?? "—"}</text>
            <Button variant="ghost" onClick={() => bevy.leave({})}>
              Отменить
            </Button>
          </>
        )}
      </node>
    </node>
  );
}
