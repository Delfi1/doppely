import type { BevyStyle, BevyTransition } from "bevy-react";

export const theme = {
  bg: "#0d0f12",
  surface: "#15181d",
  surfaceHi: "#1d2127",
  border: "#262b33",
  text: "#e6e9ef",
  muted: "#9aa3b2",
  faint: "#5d6675",
  accent: "#7aa2f7",
  accentDim: "#3d5a8a",
  danger: "#f7768e",
  ok: "#9ece6a",
};

export const font = {
  mono: "monospace",
};

export const row: BevyStyle = {
  flexDirection: "row",
  alignItems: "center",
  gap: 8,
};

export function column(gap = 8): BevyStyle {
  return { flexDirection: "column", gap };
}

export function panel(extra: BevyStyle = {}): BevyStyle {
  return {
    backgroundColor: theme.surface,
    borderRadius: 12,
    border: 1,
    borderColor: theme.border,
    padding: 14,
    ...extra,
  };
}

export function overlay(extra: BevyStyle = {}): BevyStyle {
  return {
    positionType: "absolute",
    left: 0,
    top: 0,
    width: "100%",
    height: "100%",
    justifyContent: "center",
    alignItems: "center",
    backgroundColor: "rgba(13 15 18 / 76%)",
    focusPolicy: "block",
    zIndex: 10,
    ...extra,
  };
}

export const hint: BevyStyle = {
  color: theme.faint,
  fontSize: 11,
};

export const transition: BevyTransition = {
  backgroundColor: { duration: 120, easing: "easeOut" },
  opacity: { duration: 120, easing: "easeOut" },
};
