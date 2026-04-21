export type { LexicalNode, LexicalRoot } from "@/lib/inspirationRichText/types";

export { appendParagraphTextToLexical, lexicalTextContent } from "@/lib/inspirationRichText/lexical";
export { parseLexicalJson } from "@/lib/inspirationRichText/shared";
export { renderMarkdownBlock, sanitizeEntryContent, stripMarkdownImages } from "@/lib/inspirationRichText/markdown";

import { lexicalTextContent, renderLexicalNode } from "@/lib/inspirationRichText/lexical";
import { renderMarkdownBlock, previewMarkdownContent } from "@/lib/inspirationRichText/markdown";
import { parseLexicalJson } from "@/lib/inspirationRichText/shared";

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
  return previewMarkdownContent(raw, limit);
}
