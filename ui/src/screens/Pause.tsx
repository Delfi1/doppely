import { bevy } from "../bevy";
import { Button } from "../components";
import { column, hint, panel, overlay, theme } from "../theme";

export function Pause() {
  return (
    <node style={overlay()}>
      <node style={panel({ width: 280, flexDirection: "column", gap: 14 })}>
        <text style={{ color: theme.text, fontSize: 15, fontWeight: "semibold" }}>Пауза</text>

        <node style={column(6)}>
          <Button variant="primary" onClick={() => bevy.resume({})}>
            Продолжить
          </Button>
          <Button variant="ghost" onClick={() => bevy.leave({})}>
            В главное меню
          </Button>
          <Button variant="danger" onClick={() => bevy.quit({})}>
            Закрыть игру
          </Button>
        </node>

        <text style={{ ...hint, textAlign: "center" }}>Esc — вернуться в игру</text>
      </node>
    </node>
  );
}
