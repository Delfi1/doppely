import { bevy, type LobbyChanged } from "../bevy";
import { Button } from "../components";
import { hint, panel, theme } from "../theme";

export function LobbyScreen({ lobby }: { lobby: LobbyChanged | null }) {
  const players = lobby?.players ?? 0;
  const max = lobby?.maxPlayers ?? 8;

  return (
    <node
      style={{
        flexGrow: 1,
        width: "100%",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <node style={panel({ width: 320, flexDirection: "column", gap: 14 })}>
        <text style={{ color: theme.text, fontSize: 18, fontWeight: "semibold" }}>Лобби</text>

        <node style={{ flexDirection: "column", gap: 4 }}>
          <text style={{ color: theme.text, fontSize: 13 }}>
            Игроков: {players} / {max}
          </text>
          <text style={hint}>
            {players < max ? "Ожидание игроков…" : "Все на месте"}
          </text>
        </node>

        <Button variant="primary" onClick={() => bevy.startGame({})}>
          Начать игру
        </Button>
        <Button variant="ghost" onClick={() => bevy.leave({})}>
          Выйти в меню
        </Button>
      </node>
    </node>
  );
}
