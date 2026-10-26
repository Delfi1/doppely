import { useEffect, useState, type ReactNode } from "react";
import type { BevyStyle } from "bevy-react";
import { bevy } from "./bevy";
import { theme, transition } from "./theme";

interface ButtonProps {
  children: ReactNode;
  onClick?: () => void;
  variant?: "primary" | "ghost" | "danger";
  disabled?: boolean;
  fontSize?: number;
  style?: BevyStyle;
}

const variants = {
  primary: {
    base: { backgroundColor: theme.accent, borderColor: theme.accent },
    hover: { backgroundColor: "#93b4f9" },
    press: { backgroundColor: "#6a90dd" },
    text: theme.bg,
  },
  ghost: {
    base: { backgroundColor: "transparent", borderColor: theme.border },
    hover: { backgroundColor: theme.surfaceHi, borderColor: theme.accentDim },
    press: { backgroundColor: theme.surface },
    text: theme.text,
  },
  danger: {
    base: { backgroundColor: "transparent", borderColor: theme.border },
    hover: { backgroundColor: "#2a1a20", borderColor: theme.danger },
    press: { backgroundColor: "#3a2029" },
    text: theme.danger,
  },
};

export function Button({
  children,
  onClick,
  variant = "ghost",
  disabled = false,
  fontSize = 13,
  style,
}: ButtonProps) {
  const v = variants[variant];

  return (
    <button
      onClick={disabled ? undefined : onClick}
      disabled={disabled}
      style={{
        flexDirection: "row",
        alignItems: "center",
        justifyContent: "center",
        padding: { horizontal: 14, vertical: 8 },
        border: 1,
        borderRadius: 8,
        cursor: disabled ? "default" : "pointer",
        opacity: disabled ? 0.4 : 1,
        transition,
        ...v.base,
        ...style,
      }}
      hoverStyle={disabled ? undefined : v.hover}
      pressStyle={disabled ? undefined : v.press}
      focusVisibleStyle={{ outline: { width: 2, color: theme.accent } }}
    >
      <text style={{ color: v.text, fontSize, fontWeight: "semibold" }}>{children}</text>
    </button>
  );
}

interface TextFieldProps {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  autoFocus?: boolean;
  maxLength?: number;
  style?: BevyStyle;
}

export function TextField({
  value,
  onChange,
  label: caption,
  autoFocus = false,
  maxLength,
  style,
}: TextFieldProps) {
  return (
    <node style={{ flexDirection: "column", gap: 5, ...style }}>
      {caption ? <text style={{ color: theme.muted, fontSize: 11 }}>{caption}</text> : null}
      <editableText
        value={value}
        onChange={onChange}
        maxLength={maxLength}
        autofocus={autoFocus}
        ariaLabel={caption}
        style={{
          minHeight: 18,
          width: "100%",
          padding: { horizontal: 10, vertical: 4 },
          borderRadius: 8,
          border: 1,
          borderColor: theme.border,
          backgroundColor: theme.bg,
          color: theme.text,
          fontSize: 14,
          outline: undefined,
          transition,
        }}
        hoverStyle={{ borderColor: theme.accentDim }}
        focusVisibleStyle={{ borderColor: theme.accent }}
      />
    </node>
  );
}

interface DragNumberProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
  step?: number;
  digits?: number;
  style?: BevyStyle;
}

export function DragNumber({
  label,
  value,
  onChange,
  step = 0.05,
  digits = 2,
  style,
}: DragNumberProps) {
  const [text, setText] = useState(value.toFixed(digits));
  const [drag, setDrag] = useState<{ x: number; value: number } | null>(null);

  useEffect(() => {
    if (drag) return;
    const parsed = Number(text.replace(",", "."));
    if (!Number.isFinite(parsed) || Math.abs(parsed - value) > 0.0001) {
      setText(value.toFixed(digits));
    }
    // Синхронизация только по внешнему значению, ввод пользователя не трогаем.
  }, [value]);

  const apply = (next: number) => {
    const rounded = Math.round(next * 10000) / 10000;
    setText(rounded.toFixed(digits));
    onChange(rounded);
  };

  return (
    <node
      style={{
        flexDirection: "row",
        alignItems: "center",
        gap: 2,
        flexGrow: 1,
        flexShrink: 1,
        flexBasis: 0,
        minWidth: 0,
        ...style,
      }}
    >
      <node
        onPointerDown={(event) => setDrag({ x: event.clientX, value })}
        onPointerMove={(event) => {
          if (!drag) return;
          apply(drag.value + (event.clientX - drag.x) * step);
        }}
        onPointerUp={() => setDrag(null)}
        style={{
          width: 14,
          justifyContent: "center",
          alignItems: "center",
          borderRadius: 4,
          cursor: "ew-resize",
          backgroundColor: drag ? theme.surfaceHi : "transparent",
          transition,
        }}
        hoverStyle={{ backgroundColor: theme.surfaceHi }}
      >
        <text style={{ color: drag ? theme.accent : theme.muted, fontSize: 10 }}>{label}</text>
      </node>
      <node
        style={{
          flexDirection: "row",
          alignItems: "center",
          flexGrow: 1,
          flexShrink: 1,
          flexBasis: 0,
          minWidth: 26,
          minHeight: 20,
          borderRadius: 5,
          border: 1,
          borderColor: theme.border,
          backgroundColor: theme.bg,
          padding: { horizontal: 5, vertical: 1 },
        }}
      >
        <editableText
          value={text}
          onChange={(next) => {
            setText(next);
            const parsed = Number(next.replace(",", "."));
            if (Number.isFinite(parsed)) onChange(parsed);
          }}
          maxLength={12}
          ariaLabel={label}
          style={{
            flexGrow: 1,
            flexBasis: 0,
            color: theme.text,
            fontSize: 12,
            outline: undefined,
          }}
        />
      </node>
    </node>
  );
}

export function Title({ children }: { children: ReactNode }) {
  return (
    <text style={{ color: theme.text, fontSize: 22, fontWeight: "bold", letterSpacing: 1 }}>
      {children}
    </text>
  );
}

export function useEscape(onEscape: () => void) {
  useEffect(() => {
    return bevy.on("keyDown", (e) => {
      if (e.key === "Escape") onEscape();
    });
  }, [onEscape]);
}

export function useEnter(onEnter: () => void) {
  useEffect(() => {
    return bevy.on("keyDown", (e) => {
      if (e.key === "Enter") onEnter();
    });
  }, [onEnter]);
}
