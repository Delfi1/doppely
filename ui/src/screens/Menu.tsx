import { useCallback, useState } from "react";
import { bevy, type ServerStatusWire, type ServerWire } from "../bevy";
import { Button, TextField, Title, useEnter, useEscape } from "../components";
import { column, font, hint, overlay, panel, row, theme, transition } from "../theme";

interface MenuProps {
  servers: ServerWire[];
  statuses: ServerStatusWire[];
  lastServer: string | null;
}

export function Menu({ servers, statuses, lastServer }: MenuProps) {
  const [adding, setAdding] = useState(false);

  if (servers.length === 0) {
    return (
      <node style={{ flexGrow: 1, justifyContent: "center", alignItems: "center" }}>
        <text style={{ color: theme.muted, fontSize: 13 }}>Загрузка…</text>
      </node>
    );
  }

  return (
    <node
      style={{
        flexGrow: 1,
        width: "100%",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <node style={{ width: 380, flexDirection: "column", gap: 16 }}>
        <Title>Doppely</Title>

        <node style={column(4)}>
          {servers.map((server) => (
            <ServerRow
              key={String(server.id)}
              server={server}
              active={server.address === lastServer}
              status={statuses.find((s) => s.address === server.address)}
            />
          ))}
        </node>

        <text style={hint}>Нажмите на сервер, чтобы подключиться.</text>

        <Button variant="ghost" onClick={() => setAdding(true)}>
          + Добавить сервер
        </Button>

        <node style={{ ...row, justifyContent: "flexEnd" }}>
          <Button variant="danger" onClick={() => bevy.quit({})}>
            Выйти
          </Button>
        </node>
      </node>

      {adding ? <AddServerDialog onClose={() => setAdding(false)} /> : null}
    </node>
  );
}

function ServerRow({
  server,
  active,
  status,
}: {
  server: ServerWire;
  active: boolean;
  status?: ServerStatusWire;
}) {
  const online = status?.online ?? false;

  return (
    <node
      style={{
        ...row,
        justifyContent: "spaceBetween",
        borderRadius: 10,
        border: 1,
        borderColor: active ? theme.accentDim : theme.border,
        backgroundColor: active ? theme.surface : "transparent",
        transition,
      }}
    >
      <node
        onClick={() => bevy.connect({ id: server.id })}
        style={{
          flexGrow: 1,
          flexShrink: 1,
          flexDirection: "column",
          gap: 1,
          padding: { horizontal: 12, vertical: 9 },
          borderRadius: 10,
          cursor: "pointer",
          transition,
        }}
        hoverStyle={{ backgroundColor: theme.surfaceHi }}
      >
        <text style={{ color: theme.text, fontSize: 13 }}>{server.name}</text>
        <text style={{ color: theme.faint, fontSize: 11, fontFamily: font.mono }}>
          {server.address}
        </text>
      </node>

      <node style={{ ...row, gap: 8, margin: { right: 6 } }}>
        <node
          style={{
            width: 6,
            height: 6,
            borderRadius: 999,
            backgroundColor: online ? theme.ok : theme.faint,
          }}
        />
        <text
          style={{
            color: online ? theme.muted : theme.faint,
            fontSize: 11,
            fontFamily: font.mono,
          }}
        >
          {status ? statusText(status) : "—"}
        </text>
      </node>

      {server.local ? null : (
        <Button
          variant="danger"
          fontSize={16}
          onClick={() => bevy.removeServer({ id: server.id })}
          style={{
            padding: { horizontal: 9, vertical: 2 },
            borderColor: "transparent",
            margin: { right: 6 },
          }}
        >
          ×
        </Button>
      )}
    </node>
  );
}

function statusText(status: ServerStatusWire): string {
  if (!status.online) return "оффлайн";
  const players = `${status.players}/${status.maxPlayers}`;
  if (status.mode === "game") return `${players} · в игре`;
  if (status.mode === "edit") return `${players} · редактор`;
  return players;
}

function AddServerDialog({ onClose }: { onClose: () => void }) {
  const [name, setName] = useState("");
  const [address, setAddress] = useState("");

  const submit = useCallback(() => {
    const value = address.trim();
    if (value.length === 0) return;
    bevy.addServer({ name: name.trim(), address: value });
    onClose();
  }, [address, name, onClose]);

  useEnter(submit);
  useEscape(onClose);

  return (
    <node style={overlay()}>
      <node style={panel({ width: 340, flexDirection: "column", gap: 12 })}>
        <text style={{ color: theme.text, fontSize: 15, fontWeight: "semibold" }}>
          Новый сервер
        </text>

        <TextField label="Название" value={name} onChange={setName} maxLength={24} />
        <TextField
          label="Адрес"
          value={address}
          onChange={setAddress}
          maxLength={64}
          autoFocus
        />

        <text style={hint}>Порт можно не указывать — подставится стандартный.</text>

        <node style={{ ...row, justifyContent: "flexEnd", margin: { top: 2 } }}>
          <Button variant="ghost" onClick={onClose}>
            Отмена
          </Button>
          <Button variant="primary" onClick={submit} disabled={address.trim().length === 0}>
            Добавить
          </Button>
        </node>
      </node>
    </node>
  );
}
