import type { LexicalNode } from "@/lib/inspirationRichText/types";
import { MARKDOWN_IMAGE_RE } from "@/lib/inspirationRichText/patterns";
import {
  escapeHtml,
  MAX_RENDER_DEPTH,
  parseLexicalJson,
  sanitizeHref,
  sanitizeSrc,
} from "@/lib/inspirationRichText/shared";

function applyTextFormat(text: string, format = 0) {
  let out = escapeHtml(text);
  if (format & 16) out = `<code>${out}</code>`;
  if (format & 8) out = `<u>${out}</u>`;
  if (format & 4) out = `<s>${out}</s>`;
  if (format & 2) out = `<em>${out}</em>`;
  if (format & 1) out = `<strong>${out}</strong>`;
  return out;
}

function renderTextNode(
  node: LexicalNode,
  resolveUrl: (value: string) => string,
) {
  const text = String(node.text ?? "");
  if (!text) return "";

  let last = 0;
  let html = "";
  let match: RegExpExecArray | null;
  const re = new RegExp(MARKDOWN_IMAGE_RE.source, "g");

  while ((match = re.exec(text)) !== null) {
    const start = match.index;
    if (start > last) {
      html += applyTextFormat(text.slice(last, start), Number(node.format ?? 0));
    }
    const src = sanitizeSrc(resolveUrl(String(match[1] ?? "").trim()));
    if (src) {
      html += `<img src="${escapeHtml(src)}" alt="" class="rich-inline-image" />`;
    }
    last = start + match[0].length;
  }

  if (last < text.length) {
    html += applyTextFormat(text.slice(last), Number(node.format ?? 0));
  }

  return html || applyTextFormat(text, Number(node.format ?? 0));
}

function renderChildren(
  nodes: LexicalNode[] | undefined,
  resolveUrl: (value: string) => string,
  depth: number,
) {
  if (!Array.isArray(nodes) || nodes.length === 0 || depth > MAX_RENDER_DEPTH) return "";
  return nodes.map((node) => renderLexicalNode(node, resolveUrl, depth)).join("");
}

export function renderLexicalNode(
  node: LexicalNode,
  resolveUrl: (value: string) => string,
  depth = 0,
): string {
  if (depth > MAX_RENDER_DEPTH) return "";

  const type = node.type ?? "";
  const next = depth + 1;

  if (type === "text") return renderTextNode(node, resolveUrl);
  if (type === "linebreak") return "<br />";
  if (type === "paragraph") {
    return `<p>${renderChildren(node.children, resolveUrl, next)}</p>`;
  }
  if (type === "heading") {
    const tag = node.tag === "h1" || node.tag === "h2" || node.tag === "h3" ? node.tag : "h3";
    return `<${tag}>${renderChildren(node.children, resolveUrl, next)}</${tag}>`;
  }
  if (type === "quote") {
    return `<blockquote>${renderChildren(node.children, resolveUrl, next)}</blockquote>`;
  }
  if (type === "list") {
    const tag = node.listType === "number" ? "ol" : "ul";
    return `<${tag}>${renderChildren(node.children, resolveUrl, next)}</${tag}>`;
  }
  if (type === "listitem") {
    return `<li>${renderChildren(node.children, resolveUrl, next)}</li>`;
  }
  if (type === "link") {
    const href = sanitizeHref(resolveUrl(String(node.url ?? "")));
    return `<a href="${escapeHtml(href)}" target="_blank" rel="noopener noreferrer nofollow">${renderChildren(node.children, resolveUrl, next)}</a>`;
  }
  if (type === "image") {
    const src = sanitizeSrc(resolveUrl(String(node.src ?? "")));
    if (!src) return "";
    return `<img src="${escapeHtml(src)}" alt="" class="rich-inline-image" />`;
  }

  return renderChildren(node.children, resolveUrl, next);
}

export function lexicalTextContent(input: unknown): string {
  const parsed = parseLexicalJson(input);
  if (!parsed) return "";

  const output: string[] = [];

  const collectText = (node: LexicalNode, depth = 0) => {
    if (depth > MAX_RENDER_DEPTH) return;
    if (typeof node.text === "string" && node.text.length > 0) {
      output.push(node.text);
    }
    if (node.type === "linebreak") {
      output.push("\n");
    }
    if (Array.isArray(node.children)) {
      for (const child of node.children) collectText(child, depth + 1);
      if (["paragraph", "heading", "quote", "listitem"].includes(String(node.type ?? ""))) {
        output.push("\n");
      }
    }
  };

  collectText(parsed.root);
  return output.join("").replace(/\n{3,}/g, "\n\n").trim();
}

export function appendParagraphTextToLexical(input: unknown, text: string) {
  const value = text.trim();
  const parsed =
    parseLexicalJson(input) ?? {
      root: {
        type: "root",
        format: "",
        indent: 0,
        version: 1,
        direction: null,
        children: [],
      },
    };

  if (!value) {
    return JSON.stringify(parsed);
  }

  const root = parsed.root as LexicalNode & { children: LexicalNode[] };
  if (!Array.isArray(root.children)) {
    root.children = [];
  }

  root.children.push({
    type: "paragraph",
    format: "",
    indent: 0,
    version: 1,
    direction: null,
    children: [
      {
        type: "text",
        detail: 0,
        format: 0,
        mode: "normal",
        style: "",
        text: value,
        version: 1,
      } as unknown as LexicalNode,
    ],
  } as unknown as LexicalNode);

  return JSON.stringify(parsed);
}
