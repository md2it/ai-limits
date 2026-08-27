const MONTH_NAMES = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

// Theme-specific so the fill has strong contrast against the (possibly
// brand-tinted) card behind it: light picks fully-saturated, deep tones that
// still read as red/gold/green rather than desaturating toward grey-brown;
// dark picks bright ones for contrast against dark cards. Not the shared
// --accent-* tokens (tokens.css), which stay tuned for borders/focus rings,
// not meter fills. Full saturation is deliberate here: on a light
// background, true yellow at 4.5:1 is physically impossible (its own
// luminance is too high), so the fix is to keep saturation maxed at
// whatever lightness the contrast budget allows rather than let it drift
// toward a desaturated, muddy brown. `warning` spends most of that budget —
// ~2.25:1 against white instead of 4.5:1 — to read as clean gold rather
// than amber-brown; danger and success hold the full 4.5:1 since red and
// green don't have yellow's luminance ceiling. Legibility isn't riding on
// this color alone: the adjacent "N% left" text and the bar's own filled
// length both carry the same information at full contrast.
const METER_ACCENTS = {
  light: {
    danger: [214, 14, 0],
    warning: [219, 164, 0],
    success: [0, 133, 44],
  },
  dark: {
    danger: [255, 125, 114],
    warning: [255, 207, 22],
    success: [126, 217, 65],
  },
};

function currentMeterAccents() {
  const theme = document.documentElement.dataset.theme === "dark" ? "dark" : "light";
  return METER_ACCENTS[theme];
}

// 90%+ is green, 50% is yellow, 10% or less is red; values between anchors
// are linearly interpolated so the fill is never a flat step.
export function colorForRemaining(remainingPercent) {
  const accents = currentMeterAccents();
  const clamped = Math.max(0, Math.min(100, remainingPercent));
  if (clamped <= 10) return `rgb(${accents.danger.join(", ")})`;
  if (clamped >= 90) return `rgb(${accents.success.join(", ")})`;
  const stops = [
    { pct: 10, rgb: accents.danger },
    { pct: 50, rgb: accents.warning },
    { pct: 90, rgb: accents.success },
  ];
  const upperIndex = stops.findIndex((stop) => clamped <= stop.pct);
  const upper = stops[upperIndex];
  const lower = stops[Math.max(0, upperIndex - 1)];
  const ratio = (clamped - lower.pct) / (upper.pct - lower.pct || 1);
  return `rgb(${lower.rgb.map((channel, index) => Math.round(channel + (upper.rgb[index] - channel) * ratio)).join(", ")})`;
}

const SOURCE_DISPLAY_LABELS = {
  "codex-local": "Local files",
  "codex-rpc": "RPC",
  "codex-cli": "CLI",
  "claude-rpc": "RPC",
  "claude-cli": "CLI",
  "claude-local": "Local files",
  "cursor-api2": "API2",
};

function stripTimezoneSuffix(value) {
  return String(value)
    .replace(/\s*\([^()]*\)\s*$/, "")
    .replace(/\s*UTC[+-]\d{1,2}(:\d{2})?\s*$/i, "")
    .replace(/\s*GMT[+-]\d{1,2}(:\d{2})?\s*$/i, "")
    .replace(/([+-]\d{2}:\d{2}|Z)$/, "")
    .trim();
}

export function formatTimestampForDisplay(value) {
  if (value == null || value === "") return null;
  const date = toDate(value);
  if (!date) return stripTimezoneSuffix(value);
  const time = `${pad2(date.getHours())}:${pad2(date.getMinutes())}`;
  if (isSameLocalDay(date, new Date())) return `today ${time}`;
  return `${MONTH_NAMES[date.getMonth()]} ${date.getDate()}, ${time}`;
}

function formatSourceIdLine(provider) {
  if (provider.pending) return "—";
  return `Source ${SOURCE_DISPLAY_LABELS[provider.sourceId] ?? provider.sourceId ?? "Unknown"},`;
}

function formatSourceTimestampLine(provider) {
  if (provider.pending) return "—";
  return `as of ${formatTimestampForDisplay(provider.dataTimestamp) || "unknown"}`;
}

export function formatSourceStatusLine(provider) {
  if (provider.pending) return "—";
  return `${formatSourceIdLine(provider)} ${formatSourceTimestampLine(provider)}`;
}

function formatTimeOnly(value) {
  if (value == null || value === "") return null;
  const date = toDate(value);
  if (date) return `${pad2(date.getHours())}:${pad2(date.getMinutes())}`;
  // Backend timestamps arrive pre-formatted for display (e.g. "20:03" or
  // "Aug 3, 20:03"), not as parseable instants, so fall back to pulling the
  // trailing HH:MM out of the already-formatted string.
  const match = String(value).match(/(\d{1,2}):(\d{2})\s*$/);
  return match ? `${pad2(match[1])}:${match[2]}` : null;
}

export function formatUpdateTimeLine(provider, nextRefreshAt) {
  if (provider.pending) return "—";
  const last = formatTimeOnly(provider.dataTimestamp) ?? "unknown";
  const next = formatTimeOnly(nextRefreshAt) ?? "Manual only";
  return `Last upd ${last}, next ${next}`;
}

export function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function formatDecimal(value) {
  const rounded = Math.round(Number(value) * 10) / 10;
  return Number.isInteger(rounded) ? String(rounded) : rounded.toFixed(1);
}

function toDate(value) {
  if (value instanceof Date) return Number.isNaN(value.getTime()) ? null : value;
  if (typeof value === "number") {
    const date = new Date(value < 1e12 ? value * 1000 : value);
    return Number.isNaN(date.getTime()) ? null : date;
  }
  if (typeof value !== "string" || !value.trim()) return null;
  const trimmed = value.trim();
  const numeric = /^\d+$/.test(trimmed) ? Number(trimmed) : null;
  const date = numeric == null ? new Date(trimmed) : new Date(trimmed.length > 10 ? numeric : numeric * 1000);
  return Number.isNaN(date.getTime()) ? null : date;
}

function isSameLocalDay(a, b) {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

function pad2(value) {
  return String(value).padStart(2, "0");
}

// Pulls just the plan name (e.g. "Pro", "Plus") out of the Subscription
// section's first line, for surfaces that fold the plan into the provider
// header instead of showing it as its own section — see the Popover's
// `.provider-plan-name` (docs/desktop/mac-popover.md#visual-layer). The line
// itself comes pre-formatted from the backend (plan_display_lines in
// src/presentation/sections/plan.rs) as one of: "Pro ≈ $20.00 /mo" (plan +
// price), "Plus" (plan alone), "≈ $20.00 /mo" (price alone, no plan), or
// "renews <date>" (only when neither plan nor price is present). Only the
// first two carry a plan name.
const PLAN_PRICE_SEPARATOR = " ≈";

export function extractPlanNameForHeader(plan) {
  const firstLine = plan?.lines?.[0];
  if (!firstLine || firstLine.startsWith("renews ") || firstLine.startsWith("≈")) {
    return "";
  }

  const separatorIndex = firstLine.indexOf(PLAN_PRICE_SEPARATOR);
  return separatorIndex === -1 ? firstLine : firstLine.slice(0, separatorIndex);
}
