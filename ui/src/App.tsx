import { useCallback, useEffect, useState } from "react";
import { BevyStyle } from "bevy-react";
import {
  bevy,
  type ConnectionStatusChanged,
  type HudUpdated,
  type ServerWire,
} from "./bevy";
import { Menu } from "./screens/Menu";
import { Hud } from "./screens/Hud";
import { Pause } from "./screens/Pause";
import { ConnectionScreen } from "./screens/Connection";
import { theme, transition } from "./theme";

/** Экран, которым управляет клиент (совпадает с `UiScreen::name` в Rust). */
type Screen = "menu" | "connecting" | "game" | "paused" | "error";

interface Toast {
  id: number;
  message: string;
  kind: string;
}

/**
 * Корень UI: одно React-дерево, экран выбирается по состоянию клиента.
 *
 * Клиент — единственный источник истины: он присылает `ui.screen`, `ui.servers`,
 * `ui.connection`, `ui.hud`, а UI отправляет намерения (`connect`, `addServer`, …).
 * Поэтому UI не может «разъехаться» с реальным состоянием игры.
 */
export function App() {
  const [screen, setScreen] = useState<Screen>("menu");
  const [servers, setServers] = useState<ServerWire[]>([]);
  const [connection, setConnection] = useState<ConnectionStatusChanged | null>(null);
  const [hud, setHud] = useState<HudUpdated | null>(null);
  const [toast, setToast] = useState<Toast | null>(null);

  useEffect(() => {
    const stops = [
      bevy.on("ui.screen", (e) => setScreen(e.screen as Screen)),
      bevy.on("ui.servers", (e) => setServers(e.servers)),
      bevy.on("ui.connection", (e) => setConnection(e)),
      bevy.on("ui.hud", (e) => setHud(e)),
      bevy.on("ui.toast", (e) => {
        if (e.message.length === 0) return;
        setToast({ id: Date.now(), message: e.message, kind: e.kind });
      }),
    ];
    return () => stops.forEach((stop) => stop());
  }, []);

  // Тосты сами гаснут: держим в UI-состоянии, чтобы не зависеть от Rust-таймеров.
  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), 4200);
    return () => clearTimeout(timer);
  }, [toast]);

  const busy = screen === "connecting";

  const dismiss = useCallback(() => setToast(null), []);

  return (
    <node style={rootStyle(screen)}>
      {screen === "menu" ? (
        <Menu
          servers={servers}
          lastServer={connection?.server ?? null}
          busy={busy}
        />
      ) : null}

      {screen === "connecting" ? (
        <ConnectionScreen status={connection} error={false} />
      ) : null}

      {screen === "error" ? <ConnectionScreen status={connection} error /> : null}

      {screen === "game" ? <Hud hud={hud} /> : null}
      {screen === "paused" ? <Pause /> : null}

      {toast ? <ToastView toast={toast} onDismiss={dismiss} /> : null}
    </node>
  );
}

function ToastView({ toast, onDismiss }: { toast: Toast; onDismiss: () => void }) {
  const error = toast.kind === "error";
  return (
    <node
      onClick={onDismiss}
      style={{
        positionType: "absolute",
        bottom: 20,
        alignSelf: "center",
        maxWidth: "60%",
        padding: { horizontal: 14, vertical: 10 },
        borderRadius: 10,
        border: 1,
        borderColor: error ? theme.danger : theme.accentDim,
        backgroundColor: error ? "#3a2233" : theme.surface,
        cursor: "pointer",
        transition,
      }}
    >
      <node style={{ flexDirection: "row", gap: 8, alignItems: "center" }}>
        <text style={{ color: error ? theme.danger : theme.accent, fontSize: 13 }}>
          {error ? "Ошибка" : "Инфо"}
        </text>
        <text style={{ color: theme.text, fontSize: 13 }}>{toast.message}</text>
      </node>
    </node>
  );
}

const rootStyle = (screen: Screen): BevyStyle => ({
  width: "100%",
  height: "100%",
  flexDirection: "column",
  // В игре фон прозрачный: сквозь HUD и Esc-меню должна быть видна сцена.
  backgroundColor: screen === "game" ? "transparent" : theme.bg,
});
