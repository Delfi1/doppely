import { bevy, type ConnectionStatusChanged } from "../bevy";
import { Button, Title } from "../components";
import { column, panel, row, theme } from "../theme";

/**
 * Экран подключения и экрана ошибки. Пока идёт связь — спиннер и адрес; если
 * связи нет — причина и кнопки выхода.
 */
export function ConnectionScreen({
  status,
  error,
}: {
  status: ConnectionStatusChanged | null;
  error: boolean;
}) {
  const connecting = status?.status === "connecting" && !error;

  return (
    <node
      style={{
        positionType: "absolute",
        width: "100%",
        height: "100%",
        justifyContent: "center",
        alignItems: "center",
        backgroundColor: "rgba(17 17 27 / 88%)",
      }}
    >
      <node style={panel({ width: 420, flexDirection: "column", gap: 16 })}>
        {connecting ? (
          <>
            <Title subtitle={status?.message ?? "Подключаемся…"}>Подключение</Title>
            <node style={{ ...row, gap: 10 }}>
              <Spinner />
              <text style={{ color: theme.text, fontSize: 13 }}>{status?.server ?? "—"}</text>
            </node>
            <Button variant="ghost" onClick={() => bevy.leave({})}>
              Отменить
            </Button>
          </>
        ) : (
          <>
            <Title subtitle={status?.message ?? "Соединение потеряно"}>
              Не удалось подключиться
            </Title>
            <node style={{ ...row, gap: 6 }}>
              <text style={{ color: theme.faint, fontSize: 12 }}>Сервер:</text>
              <text style={{ color: theme.text, fontSize: 12 }}>{status?.server ?? "—"}</text>
            </node>
            <node style={column(8)}>
              <Button variant="primary" onClick={() => bevy.leave({})}>
                В главное меню
              </Button>
              <Button variant="danger" onClick={() => bevy.quit({})}>
                Закрыть игру
              </Button>
            </node>
          </>
        )}
      </node>
    </node>
  );
}

function Spinner() {
  return (
    <node
      style={{
        width: 14,
        height: 14,
        borderRadius: 999,
        border: 2,
        borderColor: theme.border,
        backgroundColor: theme.accent,
      }}
    />
  );
}
