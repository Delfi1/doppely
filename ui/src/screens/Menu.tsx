import { useMemo, useState } from "react";
import { bevy, type ServerWire } from "../bevy";
import { Badge, Button, TextField, Title, useEnter } from "../components";
import { column, label, panel, row, theme, transition } from "../theme";

interface MenuProps {
  servers: ServerWire[];
  /** Адрес, к которому шло последнее подключение (для отметки «последний»). */
  lastServer: string | null;
  busy: boolean;
}

/**
 * Главное меню: список серверов (localhost всегда первый и не удаляется) и
 * форма добавления. Источник истины списка — Rust, React лишь рисует его и
 * отправляет намерения.
 */
export function Menu({ servers, lastServer, busy }: MenuProps) {
  const [name, setName] = useState("");
  const [address, setAddress] = useState("");
  const [open, setOpen] = useState(servers.length <= 1);

  const submit = () => {
    if (address.trim().length === 0) return;
    bevy.addServer({ name: name.trim(), address: address.trim() });
    setName("");
    setAddress("");
    setOpen(true);
  };
  useEnter(submit);

  const primary = useMemo(
    () => servers.find((s) => s.address === lastServer) ?? servers[0],
    [servers, lastServer],
  );

  if (!primary) {
    // Список приходит из Rust: до первого `ui.servers` рисовать нечего.
    return (
      <node style={{ flexGrow: 1, justifyContent: "center", alignItems: "center" }}>
        <text style={{ color: theme.muted, fontSize: 14 }}>
          Загрузка списка серверов…
        </text>
      </node>
    );
  }

  return (
    <node
      style={{
        flexGrow: 1,
        flexDirection: "row",
        alignItems: "center",
        justifyContent: "center",
        padding: 32,
        gap: 20,
      }}
    >
      <node style={panel({ width: 470, flexDirection: "column", gap: 14 })}>
        <Title subtitle="Асимметричная игра про станцию и её доппельгангера">
          Doppely
        </Title>

        <node style={{ ...row, gap: 8 }}>
          <Button
            variant="primary"
            disabled={busy}
            onClick={() => bevy.connect({ id: primary.id })}
          >
            Играть · {primary.name}
          </Button>
          <text style={{ color: theme.faint, fontSize: 12 }}>{primary.address}</text>
        </node>

        <Divider />

        <node style={column(8)}>
          <node style={{ ...row, justifyContent: "spaceBetween" }}>
            <text style={label}>СЕРВЕРЫ</text>
            <text style={{ color: theme.faint, fontSize: 11 }}>
              {servers.length} в списке
            </text>
          </node>

          <node
            style={{
              flexDirection: "column",
              gap: 6,
              overflowY: servers.length > 6 ? "scroll" : "visible",
              maxHeight: 260,
            }}
          >
            {servers.map((server) => (
              <ServerRow
                key={String(server.id)}
                server={server}
                last={server.address === lastServer}
                busy={busy}
              />
            ))}
          </node>
        </node>

        <Divider />

        {open ? (
          <node style={column(10)}>
            <node style={{ ...row, gap: 10, alignItems: "flexEnd" }}>
              <TextField
                value={name}
                onChange={setName}
                label="Название"
                placeholder="Домашний"
                style={{ width: 140 }}
                maxLength={24}
                onSubmit={submit}
              />
              <TextField
                value={address}
                onChange={setAddress}
                label="Адрес"
                placeholder="192.168.3.5:5000"
                style={{ flexGrow: 1 }}
                maxLength={64}
                onSubmit={submit}
              />
              <Button
                variant="ghost"
                onClick={submit}
                disabled={address.trim().length === 0}
                style={{ margin: { bottom: 1 } }}
              >
                Добавить
              </Button>
            </node>
            <text style={{ color: theme.faint, fontSize: 11 }}>
              Порт можно не указывать — подставится 5000.
            </text>
          </node>
        ) : (
          <Button variant="ghost" onClick={() => setOpen(true)}>
            + Добавить сервер
          </Button>
        )}
      </node>

      <node
        style={panel({
          width: 256,
          alignSelf: "stretch",
          flexDirection: "column",
          justifyContent: "spaceBetween",
          gap: 12,
        })}
      >
        <node style={column(10)}>
          <text style={label}>УПРАВЛЕНИЕ</text>
          <Hint k="WASD" v="движение" />
          <Hint k="Shift" v="бег" />
          <Hint k="F" v="применить к объекту" />
          <Hint k="Esc" v="меню паузы" />
        </node>

        <node style={column(8)}>
          <text style={{ color: theme.faint, fontSize: 11, lineHeight: 1.4 }}>
            Серверы хранятся в assets/servers.json. Localhost присутствует всегда.
          </text>
          <Button variant="danger" onClick={() => bevy.quit({})}>
            Выйти из игры
          </Button>
        </node>
      </node>
    </node>
  );
}

function Divider() {
  return <node style={{ height: 1, backgroundColor: theme.border }} />;
}

function Hint({ k, v }: { k: string; v: string }) {
  return (
    <node style={{ ...row, gap: 8 }}>
      <node
        style={{
          minWidth: 44,
          padding: { horizontal: 6, vertical: 2 },
          borderRadius: 6,
          border: 1,
          borderColor: theme.border,
          backgroundColor: theme.bg,
          justifyContent: "center",
        }}
      >
        <text style={{ color: theme.text, fontSize: 11, textAlign: "center" }}>{k}</text>
      </node>
      <text style={{ color: theme.muted, fontSize: 12 }}>{v}</text>
    </node>
  );
}

function ServerRow({
  server,
  last,
  busy,
}: {
  server: ServerWire;
  last: boolean;
  busy: boolean;
}) {
  return (
    <node
      style={{
        ...row,
        justifyContent: "spaceBetween",
        padding: { horizontal: 12, vertical: 9 },
        borderRadius: 9,
        border: 1,
        borderColor: last ? theme.accentDim : theme.border,
        backgroundColor: last ? "#242942" : theme.bg,
        transition,
      }}
      hoverStyle={{ borderColor: theme.accent }}
    >
      <node style={{ ...row, gap: 8, flexShrink: 1 }}>
        <node
          style={{
            width: 8,
            height: 8,
            borderRadius: 999,
            backgroundColor: last ? theme.ok : theme.faint,
          }}
        />
        <node style={{ flexDirection: "column", gap: 1 }}>
          <text style={{ color: theme.text, fontSize: 14 }}>{server.name}</text>
          <text style={{ color: theme.faint, fontSize: 11 }}>{server.address}</text>
        </node>
        {server.local ? <Badge color={theme.warn}>local</Badge> : null}
      </node>

      <node style={{ ...row, gap: 6 }}>
        <Button
          variant={last ? "primary" : "ghost"}
          disabled={busy}
          onClick={() => bevy.connect({ id: server.id })}
          style={{ padding: { horizontal: 12, vertical: 6 } }}
        >
          {last ? "Играть" : "Подключиться"}
        </Button>
        <Button
          variant="danger"
          disabled={server.local || busy}
          onClick={() => bevy.removeServer({ id: server.id })}
          style={{ padding: { horizontal: 10, vertical: 6 } }}
        >
          ✕
        </Button>
      </node>
    </node>
  );
}
