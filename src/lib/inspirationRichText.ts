export interface LexicalNode {
  type?: string;
  text?: string;
  format?: number;
  tag?: string;
  listType?: string;
  url?: string;
  src?: string;
  children?: LexicalNode[];
}

export interface LexicalRoot {
  root: LexicalNode;
}

const MARKDOWN_IMAGE_RE = /!\[[^\]]*\]\(([^)]+)\)/g;
const MARKDOWN_LINK_RE = /\[([^\]]+)\]\(([^)]+)\)/g;

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
    const src = resolveUrl(String(match[1] ?? "").trim());
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
) {
  if (!Array.isArray(nodes) || nodes.length === 0) return "";
  return nodes.map((node) => renderLexicalNode(node, resolveUrl)).join("");
}

function renderLexicalNode(
  node: LexicalNode,
  resolveUrl: (value: string) => string,
): string {
  const type = node.type ?? "";

  if (type === "text") return renderTextNode(node, resolveUrl);
  if (type === "linebreak") return "<br />";
  if (type === "paragraph") {
    return `<p>${renderChildren(node.children, resolveUrl)}</p>`;
  }
  if (type === "heading") {
    const tag = node.tag === "h1" || node.tag === "h2" || node.tag === "h3" ? node.tag : "h3";
    return `<${tag}>${renderChildren(node.children, resolveUrl)}</${tag}>`;
  }
  if (type === "quote") {
    return `<blockquote>${renderChildren(node.children, resolveUrl)}</blockquote>`;
  }
  if (type === "list") {
    const tag = node.listType === "number" ? "ol" : "ul";
    return `<${tag}>${renderChildren(node.children, resolveUrl)}</${tag}>`;
  }
  if (type === "listitem") {
    return `<li>${renderChildren(node.children, resolveUrl)}</li>`;
  }
  if (type === "link") {
    const href = resolveUrl(String(node.url ?? ""));
    return `<a href="${escapeHtml(href || "#")}" target="_blank" rel="noopener noreferrer nofollow">${renderChildren(node.children, resolveUrl)}</a>`;
  }
  if (type === "image") {
    const src = resolveUrl(String(node.src ?? ""));
    if (!src) return "";
    return `<img src="${escapeHtml(src)}" alt="" class="rich-inline-image" />`;
  }

  return renderChildren(node.children, resolveUrl);
}

export function lexicalTextContent(input: unknown): string {
  const parsed = parseLexicalJson(input);
  if (!parsed) return "";

  const output: string[] = [];

  const collectText = (node: LexicalNode) => {
    if (typeof node.text === "string" && node.text.length > 0) {
      output.push(node.text);
    }
    if (node.type === "linebreak") {
      output.push("\n");
    }
    if (Array.isArray(node.children)) {
      for (const child of node.children) collectText(child);
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

export function stripMarkdownImages(markdown: string) {
  return markdown.replace(MARKDOWN_IMAGE_RE, "");
}

function stripMarkdownForPreview(markdown: string) {
  return markdown
    .replace(/```([\s\S]*?)```/g, "$1")
    .replace(MARKDOWN_IMAGE_RE, "")
    .replace(MARKDOWN_LINK_RE, "$1")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/~~([^~]+)~~/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/__([^_]+)__/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/_([^_]+)_/g, "$1")
    .replace(/^\s{0,3}(#{1,6}\s+|>\s?|-+\s+|\*+\s+|\d+\.\s+)/gm, "")
    .replace(/^\s{0,3}([-*_]){3,}\s*$/gm, "");
}

export function sanitizeEntryContent(content: string) {
  return stripMarkdownImages(content)
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function applyInlineMarkdown(
  value: string,
  resolveUrl: (value: string) => string,
) {
  let html = escapeHtml(value);
  html = html.replace(/`([^`]+)`/g, "<code>$1</code>");
  html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, label: string, href: string) => {
    const resolvedHref = escapeHtml(resolveUrl(href));
    return `<a href="${resolvedHref}" target="_blank" rel="noreferrer">${label}</a>`;
  });
  return html;
}

export function renderMarkdownBlock(
  content: string,
  resolveUrl: (value: string) => string,
) {
  const normalized = sanitizeEntryContent(content);
  if (!normalized) return "";

  const lines = normalized.split("\n");
  const html: string[] = [];
  let inList = false;
  let inCode = false;
  let paragraph: string[] = [];

  const flushParagraph = () => {
    if (!paragraph.length) return;
    html.push(`<p>${applyInlineMarkdown(paragraph.join("<br />"), resolveUrl)}</p>`);
    paragraph = [];
  };

  const closeList = () => {
    if (!inList) return;
    html.push("</ul>");
    inList = false;
  };

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();
    const trimmed = line.trim();

    if (trimmed.startsWith("```")) {
      closeList();
      flushParagraph();
      if (inCode) {
        html.push("</code></pre>");
        inCode = false;
      } else {
        html.push("<pre><code>");
        inCode = true;
      }
      continue;
    }

    if (inCode) {
      html.push(`${escapeHtml(rawLine)}\n`);
      continue;
    }

    if (!trimmed) {
      closeList();
      flushParagraph();
      continue;
    }

    const headingMatch = trimmed.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      closeList();
      flushParagraph();
      const level = headingMatch[1].length;
      html.push(`<h${level}>${applyInlineMarkdown(headingMatch[2], resolveUrl)}</h${level}>`);
      continue;
    }

    const quoteMatch = trimmed.match(/^>\s?(.*)$/);
    if (quoteMatch) {
      closeList();
      flushParagraph();
      html.push(`<blockquote>${applyInlineMarkdown(quoteMatch[1], resolveUrl)}</blockquote>`);
      continue;
    }

    const listMatch = trimmed.match(/^[-*]\s+(.+)$/);
    if (listMatch) {
      flushParagraph();
      if (!inList) {
        html.push("<ul>");
        inList = true;
      }
      html.push(`<li>${applyInlineMarkdown(listMatch[1], resolveUrl)}</li>`);
      continue;
    }

    closeList();
    paragraph.push(trimmed);
  }

  closeList();
  flushParagraph();

  if (inCode) html.push("</code></pre>");

  return html.join("");
}

export function renderInspirationContentHtml(
  content: string,
  contentLexical: string | null | undefined,
  resolveUrl: (value: string) => string,
) {
  const lexical = parseLexicalJson(contentLexical);
  if (lexical) {
    const children = Array.isArray(lexical.root.children) ? lexical.root.children : [];
    return children.map((node) => renderLexicalNode(node, resolveUrl)).join("");
  }
  return renderMarkdownBlock(content, resolveUrl);
}

export function previewInspirationContent(
  content: string,
  contentLexical: string | null | undefined,
  limit = 180,
) {
  const raw = lexicalTextContent(contentLexical) || content;
  const normalized = stripMarkdownForPreview(raw).replace(/\s+/g, " ").trim();
  if (!normalized) return "";
  if (normalized.length <= limit) return normalized;
  return `${normalized.slice(0, limit).trimEnd()}...`;
}
