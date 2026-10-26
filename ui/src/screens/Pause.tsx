import { bevy } from "../bevy";
import { Button, Title } from "../components";
import { column, panel, row, theme } from "../theme";

/** Esc-меню: продолжение, выход в главное меню и выход из игры. */
export function Pause() {
  return (
    <node
      style={{
        positionType: "absolute",
        width: "100%",
        height: "100%",
        justifyContent: "center",
        alignItems: "center",
        // Затемнение позади меню.
        backgroundColor: "rgba(17 17 27 / 72%)",
      }}
    >
      <node style={panel({ width: 360, flexDirection: "column", gap: 16 })}>
        <Title subtitle="Игра приостановлена">Пауза</Title>

        <node style={column(8)}>
          <Button variant="primary" onClick={() => bevy.resume({})}>
            Продолжить
          </Button>
          <Button variant="ghost" onClick={() => bevy.leave({})}>
            Выйти в главное меню
          </Button>
          <Button variant="danger" onClick={() => bevy.quit({})}>
            Закрыть игру
          </Button>
        </node>

        <node style={{ ...row, gap: 6, justifyContent: "center" }}>
          <text style={{ color: theme.faint, fontSize: 11 }}>
            Esc — вернуться в игру
          </text>
        </node>
      </node>
    </node>
  );
}
