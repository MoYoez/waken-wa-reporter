import { MARKDOWN_IMAGE_RE, MARKDOWN_LINK_RE } from "@/lib/inspirationRichText/patterns";
import { escapeHtml, sanitizeHref } from "@/lib/inspirationRichText/shared";

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
    const resolvedHref = sanitizeHref(resolveUrl(href));
    return `<a href="${escapeHtml(resolvedHref)}" target="_blank" rel="noopener noreferrer nofollow">${label}</a>`;
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

export function previewMarkdownContent(raw: string, limit = 180) {
  const normalized = stripMarkdownForPreview(raw).replace(/\s+/g, " ").trim();
  if (!normalized) return "";
  if (normalized.length <= limit) return normalized;
  return `${normalized.slice(0, limit).trimEnd()}...`;
}
