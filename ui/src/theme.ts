// Визуальный язык UI: тёмная палитра, шрифты и общие стили.
// bevy-react принимает CSS-подобные значения, поэтому стили читаются как
// обычные инлайн-стили — без CSS-файлов и классов.

import type { BevyStyle, BevyTransition } from "bevy-react";

export const theme = {
  bg: "#11111b",
  surface: "#1e1e2e",
  surfaceHi: "#313244",
  border: "#313244",
  text: "#cdd6f4",
  muted: "#9399b2",
  faint: "#6c7086",
  accent: "#89b4fa",
  accentDim: "#4a6fae",
  danger: "#f38ba8",
  ok: "#a6e3a1",
  warn: "#f9e2af",
};

export const font = {
  mono: "monospace",
};

/** Экран на весь размер окна. */
export const fullScreen: BevyStyle = {
  width: "100%",
  height: "100%",
  padding: 32,
};

/** Колонка с центрированием по обеим осям. */
export const centered: BevyStyle = {
  ...fullScreen,
  flexDirection: "column",
  justifyContent: "center",
  alignItems: "center",
  gap: 18,
};

/** Строка с выравниванием по центру по вертикали. */
export const row: BevyStyle = {
  flexDirection: "row",
  alignItems: "center",
  gap: 10,
};

/** Колонка с произвольным gap. */
export function column(gap = 10): BevyStyle {
  return { flexDirection: "column", gap };
}

/** Панель с рамкой и скруглением. */
export function panel(extra: BevyStyle = {}): BevyStyle {
  return {
    backgroundColor: theme.surface,
    borderRadius: 14,
    border: 1,
    borderColor: theme.border,
    padding: 20,
    ...extra,
  };
}

/** Подпись над полем/списком. */
export const label: BevyStyle = {
  color: theme.muted,
  fontSize: 12,
  fontWeight: "semibold",
  letterSpacing: 1,
};

export const transition: BevyTransition = {
  backgroundColor: { duration: 140, easing: "easeOut" },
  opacity: { duration: 140, easing: "easeOut" },
};
