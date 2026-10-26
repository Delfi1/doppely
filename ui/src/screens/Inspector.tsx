import { bevy, type SceneChanged, type SceneComponent } from "../bevy";
import { Button, DragNumber } from "../components";
import { hint, panel, row, theme, transition } from "../theme";

type Kind = "dynamic" | "glow" | "light" | "friction";

type Object = SceneChanged["objects"][number];

const allKinds: Kind[] = ["dynamic", "glow", "light", "friction"];

function kindOf(component: SceneComponent): Kind {
  if (typeof component === "string") return component;
  return Object.keys(component)[0] as Kind;
}

function hasKind(components: SceneComponent[], kind: Kind): boolean {
  return components.some((component) => kindOf(component) === kind);
}

function asLight(component: SceneComponent): { intensity: number; radius: number } | null {
  return typeof component === "object" && "light" in component ? component.light : null;
}

function asFriction(component: SceneComponent): { coefficient: number } | null {
  return typeof component === "object" && "friction" in component ? component.friction : null;
}

function newComponent(kind: Kind): SceneComponent {
  switch (kind) {
    case "dynamic":
      return "dynamic";
    case "glow":
      return "glow";
    case "light":
      return { light: { intensity: 100000, radius: 10 } };
    case "friction":
      return { friction: { coefficient: 0.8 } };
  }
}

function axis(triple: number[], index: number, value: number): [number, number, number] {
  return triple.map((item, i) => (i === index ? value : item)) as [number, number, number];
}

export function Inspector({ scene }: { scene: SceneChanged | null }) {
  const objects = scene?.objects ?? [];
  const selected = scene?.selected ?? [];
  const primaryId = selected.length > 0 ? selected[selected.length - 1] : null;
  const object = objects.find((item) => item.id === primaryId) ?? null;

  return (
    <node
      style={{
        positionType: "absolute",
        right: 16,
        top: 16,
        width: 264,
        flexDirection: "column",
        gap: 8,
      }}
    >
      <SceneInspector objects={objects} selected={selected} />
      <EntityInspector object={object} selected={selected} />
    </node>
  );
}

function SceneInspector({ objects, selected }: { objects: Object[]; selected: number[] }) {
  return (
    <node style={panel({ flexDirection: "column", gap: 6, focusPolicy: "pass" })}>
      <node style={{ ...row, justifyContent: "spaceBetween" }}>
        <text style={{ color: theme.text, fontSize: 13, fontWeight: "semibold" }}>Сцена</text>
        <text style={hint}>
          {selected.length > 1 ? `${selected.length} / ${objects.length}` : `${objects.length} объектов`}
        </text>
      </node>

      <node
        style={{
          flexDirection: "column",
          gap: 2,
          maxHeight: 150,
          overflowY: objects.length > 6 ? "scroll" : "visible",
        }}
      >
        {objects.map((item) => {
          const active = selected.includes(item.id);
          const primary = active && selected[selected.length - 1] === item.id;
          return (
            <node
              key={String(item.id)}
              onClick={() => bevy.selectObject({ id: item.id })}
              style={{
                ...row,
                justifyContent: "spaceBetween",
                gap: 6,
                padding: { horizontal: 6, vertical: 3 },
                borderRadius: 5,
                cursor: "pointer",
                border: 1,
                borderColor: primary ? theme.accentDim : "transparent",
                backgroundColor: active ? theme.surfaceHi : "transparent",
                transition,
              }}
              hoverStyle={{ backgroundColor: theme.surfaceHi }}
            >
              <text style={{ color: active ? theme.text : theme.muted, fontSize: 11 }}>
                #{item.id} {item.name}
              </text>
              <text style={{ ...hint, fontFamily: "monospace" }}>
                {item.position[0].toFixed(1)} {item.position[1].toFixed(1)}{" "}
                {item.position[2].toFixed(1)}
              </text>
            </node>
          );
        })}
      </node>

      <text style={hint}>Shift + клик — добавить к выбору, Ctrl+Z / Ctrl+Y — история.</text>
    </node>
  );
}

function EntityInspector({ object, selected }: { object: Object | null; selected: number[] }) {
  if (!object) {
    return (
      <node style={panel({ flexDirection: "column", gap: 6, focusPolicy: "pass" })}>
        <text style={{ color: theme.text, fontSize: 13, fontWeight: "semibold" }}>Сущность</text>
        <text style={hint}>Клик по объекту в сцене или в списке — выбор.</text>
      </node>
    );
  }

  const setComponents = (components: SceneComponent[]) =>
    bevy.setComponents({ id: object.id, components });

  const apply = (
    position: [number, number, number],
    rotation: [number, number, number],
    size: [number, number, number],
  ) =>
    bevy.setTransform({
      id: object.id,
      position,
      rotation,
      size: size.map((value) => Math.max(0.02, value)) as [number, number, number],
    });

  const replace = (index: number, component: SceneComponent) =>
    setComponents(object.components.map((item, i) => (i === index ? component : item)));

  const remove = () => {
    const ids = selected.length > 0 ? selected : [object.id];
    bevy.removeObjects({ ids });
  };

  return (
    <node style={panel({ flexDirection: "column", gap: 7, focusPolicy: "pass" })}>
      <node style={{ ...row, justifyContent: "spaceBetween" }}>
        <text style={{ color: theme.text, fontSize: 13, fontWeight: "semibold" }}>Сущность</text>
        <text style={hint}>
          {object.kind} · {object.size.map((value) => value.toFixed(1)).join("×")}
        </text>
      </node>

      <text style={{ color: theme.text, fontSize: 11 }}>
        #{object.id} {object.name}
        {selected.length > 1 ? ` · выбрано ${selected.length}` : ""}
      </text>

      <text style={hint}>Позиция</text>

      <node style={{ ...row, gap: 4 }}>
        <DragNumber
          label="X"
          value={object.position[0]}
          onChange={(value) =>
            apply(axis(object.position, 0, value), object.rotation, object.size)
          }
        />
        <DragNumber
          label="Y"
          value={object.position[1]}
          onChange={(value) =>
            apply(axis(object.position, 1, value), object.rotation, object.size)
          }
        />
        <DragNumber
          label="Z"
          value={object.position[2]}
          onChange={(value) =>
            apply(axis(object.position, 2, value), object.rotation, object.size)
          }
        />
      </node>

      <text style={hint}>Поворот, °</text>

      <node style={{ ...row, gap: 4 }}>
        <DragNumber
          label="X"
          value={object.rotation[0]}
          step={1}
          digits={1}
          onChange={(value) => apply(object.position, axis(object.rotation, 0, value), object.size)}
        />
        <DragNumber
          label="Y"
          value={object.rotation[1]}
          step={1}
          digits={1}
          onChange={(value) => apply(object.position, axis(object.rotation, 1, value), object.size)}
        />
        <DragNumber
          label="Z"
          value={object.rotation[2]}
          step={1}
          digits={1}
          onChange={(value) => apply(object.position, axis(object.rotation, 2, value), object.size)}
        />
      </node>

      <text style={hint}>Масштаб</text>

      <node style={{ ...row, gap: 4 }}>
        <DragNumber
          label="X"
          value={object.size[0]}
          onChange={(value) => apply(object.position, object.rotation, axis(object.size, 0, value))}
        />
        <DragNumber
          label="Y"
          value={object.size[1]}
          onChange={(value) => apply(object.position, object.rotation, axis(object.size, 1, value))}
        />
        <DragNumber
          label="Z"
          value={object.size[2]}
          onChange={(value) => apply(object.position, object.rotation, axis(object.size, 2, value))}
        />
      </node>

      <text style={hint}>Компоненты</text>

      {object.components.map((component, index) => {
        const kind = kindOf(component);
        const light = asLight(component);
        const friction = asFriction(component);

        return (
          <node key={`${kind}-${index}`} style={{ ...row, gap: 4 }}>
            <text style={{ color: theme.text, fontSize: 11, width: 52 }}>{kind}</text>

            <node style={{ ...row, gap: 3, flexGrow: 1, flexShrink: 1, flexBasis: 0 }}>
              {light ? (
                <>
                  <DragNumber
                    label="I"
                    value={light.intensity}
                    step={500}
                    digits={0}
                    onChange={(value) =>
                      replace(index, {
                        light: { intensity: Math.max(0, value), radius: light.radius },
                      })
                    }
                  />
                  <DragNumber
                    label="R"
                    value={light.radius}
                    step={0.2}
                    onChange={(value) =>
                      replace(index, {
                        light: { intensity: light.intensity, radius: Math.max(0.1, value) },
                      })
                    }
                  />
                </>
              ) : null}

              {friction ? (
                <DragNumber
                  label="μ"
                  value={friction.coefficient}
                  step={0.01}
                  onChange={(value) =>
                    replace(index, { friction: { coefficient: Math.max(0, value) } })
                  }
                />
              ) : null}
            </node>

            <Button
              variant="danger"
              fontSize={14}
              onClick={() => setComponents(object.components.filter((_, i) => i !== index))}
              style={{ padding: { horizontal: 6, vertical: 1 }, borderColor: "transparent" }}
            >
              ×
            </Button>
          </node>
        );
      })}

      <node style={{ ...row, gap: 4, flexWrap: "wrap" }}>
        {allKinds
          .filter((kind) => !hasKind(object.components, kind))
          .map((kind) => (
            <node
              key={kind}
              onClick={() => setComponents([...object.components, newComponent(kind)])}
              style={{
                padding: { horizontal: 6, vertical: 2 },
                borderRadius: 5,
                border: 1,
                borderColor: theme.border,
                cursor: "pointer",
                transition,
              }}
              hoverStyle={{ borderColor: theme.accentDim, backgroundColor: theme.surfaceHi }}
            >
              <text style={{ color: theme.muted, fontSize: 10 }}>+ {kind}</text>
            </node>
          ))}
      </node>

      <Button variant="danger" onClick={remove}>
        {selected.length > 1 ? `Удалить ${selected.length}` : "Удалить объект"}
      </Button>
    </node>
  );
}
