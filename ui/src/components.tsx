import { useEffect, useState, type ReactNode } from "react";
import { BevyStyle } from "bevy-react";
import { bevy } from "./bevy";
import { font, label, row, theme, transition } from "./theme";

interface ButtonProps {
  children: ReactNode;
  onClick?: () => void;
  variant?: "primary" | "ghost" | "danger";
  disabled?: boolean;
  style?: BevyStyle;
}

const variants = {
  primary: {
    base: { backgroundColor: theme.accent, borderColor: theme.accent },
    hover: { backgroundColor: "#a3c8ff" },
    press: { backgroundColor: "#6d94d6" },
    text: theme.bg,
  },
  ghost: {
    base: { backgroundColor: theme.surfaceHi, borderColor: theme.border },
    hover: { backgroundColor: "#3b3f5c", borderColor: theme.accentDim },
    press: { backgroundColor: "#2a2d45" },
    text: theme.text,
  },
  danger: {
    base: { backgroundColor: "transparent", borderColor: theme.danger },
    hover: { backgroundColor: "#3a2233" },
    press: { backgroundColor: "#4d2535" },
    text: theme.danger,
  },
};

/** Кнопка с hover/press-состояниями и плавным переходом. */
export function Button({
  children,
  onClick,
  variant = "ghost",
  disabled = false,
  style,
}: ButtonProps) {
  const v = variants[variant];

  return (
    <button
      onClick={disabled ? undefined : onClick}
      disabled={disabled}
      style={{
        ...row,
        justifyContent: "center",
        padding: { horizontal: 16, vertical: 9 },
        border: 1,
        borderRadius: 9,
        cursor: disabled ? "default" : "pointer",
        opacity: disabled ? 0.45 : 1,
        transition,
        ...v.base,
        ...style,
      }}
      hoverStyle={disabled ? undefined : v.hover}
      pressStyle={disabled ? undefined : v.press}
      focusVisibleStyle={{ outline: { width: 2, color: theme.accent } }}
    >
      <text style={{ color: v.text, fontSize: 14, fontWeight: "semibold" }}>
        {children}
      </text>
    </button>
  );
}

interface TextFieldProps {
  value: string;
  onChange: (value: string) => void;
  /** Показать вместо значения, пока оно пустое. */
  placeholder?: string;
  label?: string;
  onSubmit?: () => void;
  style?: BevyStyle;
  maxLength?: number;
}

/** Поле ввода с подписью-подсказкой; Enter вызывает `onSubmit`. */
export function TextField({
  value,
  onChange,
  placeholder,
  label: caption,
  onSubmit,
  style,
  maxLength,
}: TextFieldProps) {
  return (
    <node style={{ flexDirection: "column", gap: 6, ...style }}>
      {caption ? <text style={label}>{caption}</text> : null}
      <node
        style={{
          flexDirection: "row",
          alignItems: "center",
          borderRadius: 9,
          border: 1,
          borderColor: theme.border,
          backgroundColor: theme.bg,
          padding: { horizontal: 12, vertical: 6 },
          transition,
        }}
      >
        {value.length === 0 && placeholder ? (
          <text
            style={{
              positionType: "absolute",
              left: 12,
              color: theme.faint,
              fontSize: 14,
            }}
          >
            {placeholder}
          </text>
        ) : null}
        <editableText
          value={value}
          onChange={onChange}
          maxLength={maxLength}
          ariaLabel={caption ?? placeholder}
          style={{
            flexGrow: 1,
            color: theme.text,
            fontSize: 14,
            outline: undefined,
          }}
        />
      </node>
    </node>
  );
}

/**
 * Ловит Enter, не зная, какое поле в фокусе: игрок либо что-то печатает, либо
 * нет. Решение принимает вызывающая сторона.
 */
export function useEnter(onEnter: () => void) {
  useEffect(() => {
    const stop = bevy.on("keyDown", (e) => {
      if (e.key === "Enter") onEnter();
    });
    return stop;
    // Подписка переезжает при смене обработчика, чтобы всегда видеть актуальный.
  }, [onEnter]);
}

/** Заголовок экрана. */
export function Title({ children, subtitle }: { children: ReactNode; subtitle?: string }) {
  return (
    <node style={{ flexDirection: "column", gap: 4 }}>
      <text
        style={{
          color: theme.text,
          fontSize: 30,
          fontWeight: "black",
          letterSpacing: 1,
        }}
      >
        {children}
      </text>
      {subtitle ? (
        <text style={{ color: theme.muted, fontSize: 13 }}>{subtitle}</text>
      ) : null}
    </node>
  );
}

/** Компактная метка-статус. */
export function Badge({
  children,
  color = theme.accent,
}: {
  children: ReactNode;
  color?: string;
}) {
  return (
    <node
      style={{
        padding: { horizontal: 8, vertical: 3 },
        borderRadius: 999,
        backgroundColor: theme.bg,
        border: 1,
        borderColor: color,
      }}
    >
      <text style={{ color, fontSize: 11, fontFamily: font.mono }}>{children}</text>
    </node>
  );
}

/** Двусторонний инпут: локальное значение + изменение в родителе. */
export function useField(initial = "") {
  const [value, setValue] = useState(initial);
  return { value, setValue };
}
