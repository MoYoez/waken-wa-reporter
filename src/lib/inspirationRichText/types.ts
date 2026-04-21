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
