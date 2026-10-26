import { useCallback, useEffect, useState } from "react";
import type { BevyStyle } from "bevy-react";
import {
  bevy,
  type ConnectionStatusChanged,
  type EditorChanged,
  type HudUpdated,
  type LobbyChanged,
  type SceneChanged,
  type ServerStatusWire,
  type ServerWire,
} from "./bevy";
import { Menu } from "./screens/Menu";
import { LobbyScreen } from "./screens/Lobby";
import { EditorScreen } from "./screens/Editor";
import { Inspector } from "./screens/Inspector";
import { Hud } from "./screens/Hud";
import { Pause } from "./screens/Pause";
import { ConnectionScreen } from "./screens/Connection";
import { theme, transition } from "./theme";

type Screen = "menu" | "connecting" | "lobby" | "edit" | "game" | "paused" | "error";

interface Toast {
  id: number;
  message: string;
  kind: string;
}

export function App() {
  const [screen, setScreen] = useState<Screen>("menu");
  const [servers, setServers] = useState<ServerWire[]>([]);
  const [statuses, setStatuses] = useState<ServerStatusWire[]>([]);
  const [connection, setConnection] = useState<ConnectionStatusChanged | null>(null);
  const [lobby, setLobby] = useState<LobbyChanged | null>(null);
  const [editor, setEditor] = useState<EditorChanged | null>(null);
  const [scene, setScene] = useState<SceneChanged | null>(null);
  const [hud, setHud] = useState<HudUpdated | null>(null);
  const [toast, setToast] = useState<Toast | null>(null);

  useEffect(() => {
    const stops = [
      bevy.on("ui.screen", (e) => setScreen(e.screen as Screen)),
      bevy.on("ui.servers", (e) => setServers(e.servers)),
      bevy.on("ui.status", (e) => setStatuses(e.statuses)),
      bevy.on("ui.connection", (e) => setConnection(e)),
      bevy.on("ui.lobby", (e) => setLobby(e)),
      bevy.on("ui.editor", (e) => setEditor(e)),
      bevy.on("ui.scene", (e) => setScene(e)),
      bevy.on("ui.hud", (e) => setHud(e)),
      bevy.on("ui.toast", (e) => {
        if (e.message.length === 0) return;
        setToast({ id: Date.now(), message: e.message, kind: e.kind });
      }),
    ];
    return () => stops.forEach((stop) => stop());
  }, []);

  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), 4200);
    return () => clearTimeout(timer);
  }, [toast]);

  const dismiss = useCallback(() => setToast(null), []);

  return (
    <node style={rootStyle(screen)}>
      {screen === "menu" ? (
        <Menu servers={servers} statuses={statuses} lastServer={connection?.server ?? null} />
      ) : null}

      {screen === "connecting" ? <ConnectionScreen status={connection} error={false} /> : null}
      {screen === "error" ? <ConnectionScreen status={connection} error /> : null}

      {screen === "lobby" ? <LobbyScreen lobby={lobby} /> : null}
      {screen === "edit" ? <EditorScreen editor={editor} /> : null}
      {screen === "edit" ? <Inspector scene={scene} /> : null}

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
      style={{
        positionType: "absolute",
        left: 0,
        bottom: 24,
        width: "100%",
        justifyContent: "center",
      }}
    >
      <node
        onClick={onDismiss}
        style={{
          maxWidth: 480,
          padding: { horizontal: 12, vertical: 8 },
          borderRadius: 8,
          border: 1,
          borderColor: error ? theme.danger : theme.border,
          backgroundColor: theme.surface,
          cursor: "pointer",
          transition,
        }}
      >
        <text style={{ color: error ? theme.danger : theme.text, fontSize: 12 }}>
          {toast.message}
        </text>
      </node>
    </node>
  );
}

const rootStyle = (screen: Screen): BevyStyle => ({
  width: "100%",
  height: "100%",
  flexDirection: "column",
  backgroundColor:
    screen === "game" || screen === "paused" || screen === "edit" ? "transparent" : theme.bg,
});
