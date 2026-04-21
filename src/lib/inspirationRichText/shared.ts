import type { LexicalRoot } from "@/lib/inspirationRichText/types";

export const MAX_RENDER_DEPTH = 32;

function isObject(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function isLexicalRoot(value: unknown): value is LexicalRoot {
  if (!isObject(value)) return false;
  const root = value.root;
  return isObject(root) && Array.isArray(root.children);
}

export function parseLexicalJson(input: unknown): LexicalRoot | null {
  if (!input) return null;
  if (typeof input === "string") {
    const raw = input.trim();
    if (!raw) return null;
    try {
      const parsed: unknown = JSON.parse(raw);
      return isLexicalRoot(parsed) ? parsed : null;
    } catch {
      return null;
    }
  }
  return isLexicalRoot(input) ? input : null;
}

export function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function isSafeUrl(url: string): boolean {
  const trimmed = url.trim();
  if (!trimmed) return false;
  if (/^https?:\/\//i.test(trimmed)) return true;
  if (trimmed.startsWith("/") || trimmed.startsWith("./") || trimmed.startsWith("../")) return true;
  if (/^data:image\//i.test(trimmed)) return true;
  return false;
}

export function sanitizeHref(url: string): string {
  return isSafeUrl(url) ? url : "#";
}

export function sanitizeSrc(url: string): string {
  return isSafeUrl(url) ? url : "";
}
