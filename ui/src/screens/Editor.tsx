import { useEffect, useState } from "react";
import { bevy, type EditorChanged, type ObjectType } from "../bevy";
import { Button, TextField, useEnter } from "../components";
import { hint, panel, row, theme, transition } from "../theme";

const spawnables: { kind: ObjectType; label: string }[] = [
  { kind: "Cuboid", label: "Куб" },
  { kind: "Sphere", label: "Сфера" },
  { kind: "Capsule", label: "Капсула" },
  { kind: "Cylinder", label: "Цилиндр" },
  { kind: "Cone", label: "Конус" },
  { kind: "Spawnpoint", label: "Точка спавна" },
];

interface Picker {
  x: number;
  y: number;
  position: [number, number, number];
}

export function EditorScreen({ editor }: { editor: EditorChanged | null }) {
  const [name, setName] = useState("");
  const [picker, setPicker] = useState<Picker | null>(null);
  const [windowSize, setWindowSize] = useState<{ width: number; height: number } | null>(null);

  const scene = editor?.scene ?? "";
  const scenes = editor?.scenes ?? [];
  const message = editor?.message ?? "";
  const ok = editor?.ok ?? true;
  const canUndo = editor?.canUndo ?? false;
  const canRedo = editor?.canRedo ?? false;
  const hasName = name.trim().length > 0;

  useEffect(() => {
    return bevy.on("ui.picker", (event) =>
      setPicker({ x: event.x, y: event.y, position: event.position }),
    );
  }, []);

  useEffect(() => {
    bevy.window
      .size()
      .then((size) => setWindowSize(size))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!picker) return;
    return bevy.on("keyDown", (event) => {
      if (event.key === "Escape") setPicker(null);
    });
  }, [picker]);

  const submit = () => {
    const value = name.trim();
    if (value.length === 0) return;
    bevy.saveScene({ name: value });
  };
  useEnter(submit);

  const spawn = (kind: ObjectType) => {
    if (!picker) return;
    bevy.spawnObject({ kind, position: picker.position });
    setPicker(null);
  };

  const pickerLeft = picker
    ? Math.min(picker.x + 6, (windowSize?.width ?? 1280) - 180)
    : 0;
  const pickerTop = picker
    ? Math.min(picker.y + 6, (windowSize?.height ?? 720) - 220)
    : 0;

  return (
    <node style={{ positionType: "absolute", left: 16, top: 16, width: 300 }}>
      <node style={panel({ flexDirection: "column", gap: 9, focusPolicy: "pass" })}>
        <node style={{ ...row, justifyContent: "spaceBetween" }}>
          <text style={{ color: theme.text, fontSize: 15, fontWeight: "semibold" }}>Редактор</text>
          <text style={hint}>{scene ? scene : "по умолчанию"}</text>
        </node>

        <text style={hint}>
          ЛКМ — выбрать · Shift+ЛКМ — мультивыбор · тянуть — двигать · F — меню создания ·
          Del — удалить · Ctrl+Z / Ctrl+Y — отменить и повторить · ПКМ — обзор · WASD/Space/Ctrl —
          полёт
        </text>

        <TextField label="Имя сцены" value={name} onChange={setName} maxLength={32} />

        <node style={{ ...row, gap: 6 }}>
          <Button variant="primary" onClick={submit} disabled={!hasName} style={{ flexGrow: 1 }}>
            Сохранить
          </Button>
          <Button
            variant="ghost"
            onClick={() => bevy.loadScene({ name: name.trim() })}
            disabled={!hasName}
            style={{ flexGrow: 1 }}
          >
            Загрузить
          </Button>
        </node>

        <node style={{ ...row, gap: 6 }}>
          <Button
            variant="ghost"
            onClick={() => bevy.undo({})}
            disabled={!canUndo}
            style={{ flexGrow: 1 }}
          >
            Отменить
          </Button>
          <Button
            variant="ghost"
            onClick={() => bevy.redo({})}
            disabled={!canRedo}
            style={{ flexGrow: 1 }}
          >
            Повторить
          </Button>
        </node>

        {message ? (
          <text style={{ color: ok ? theme.ok : theme.danger, fontSize: 11 }}>{message}</text>
        ) : null}

        <node style={{ ...row, justifyContent: "spaceBetween" }}>
          <text style={hint}>Сцены: {scenes.length > 0 ? scenes.length : "нет"}</text>
          <node
            onClick={() => bevy.refreshScenes({})}
            style={{
              padding: { horizontal: 6, vertical: 2 },
              borderRadius: 6,
              cursor: "pointer",
              transition,
            }}
            hoverStyle={{ backgroundColor: theme.surfaceHi }}
          >
            <text style={hint}>обновить</text>
          </node>
        </node>

        <node
          style={{
            flexDirection: "column",
            gap: 2,
            maxHeight: 160,
            overflowY: scenes.length > 6 ? "scroll" : "visible",
          }}
        >
          {scenes.map((item) => (
            <node
              key={item}
              onClick={() => bevy.loadScene({ name: item })}
              style={{
                ...row,
                justifyContent: "spaceBetween",
                padding: { horizontal: 8, vertical: 5 },
                borderRadius: 6,
                cursor: "pointer",
                backgroundColor: item === scene ? theme.surfaceHi : "transparent",
                transition,
              }}
              hoverStyle={{ backgroundColor: theme.surfaceHi }}
            >
              <text style={{ color: theme.text, fontSize: 12 }}>{item}</text>
              <text style={hint}>загрузить</text>
            </node>
          ))}
        </node>

        <Button variant="ghost" onClick={() => bevy.leave({})}>
          Выйти в меню
        </Button>
      </node>

      {picker ? (
        <node
          style={{
            positionType: "absolute",
            left: pickerLeft,
            top: pickerTop,
            width: 168,
          }}
        >
          <node style={panel({ flexDirection: "column", gap: 2, padding: 6 })}>
            {spawnables.map((item) => (
              <node
                key={item.kind}
                onClick={() => spawn(item.kind)}
                style={{
                  padding: { horizontal: 8, vertical: 4 },
                  borderRadius: 5,
                  cursor: "pointer",
                  transition,
                }}
                hoverStyle={{ backgroundColor: theme.surfaceHi }}
              >
                <text style={{ color: theme.text, fontSize: 12 }}>{item.label}</text>
              </node>
            ))}
          </node>
        </node>
      ) : null}
    </node>
  );
}
