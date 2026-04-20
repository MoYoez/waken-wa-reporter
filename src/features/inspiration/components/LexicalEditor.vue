<script setup lang="ts">
import { computed } from "vue";
import { LexicalComposer } from "lexical-vue/LexicalComposer";
import { LinkNode } from "@lexical/link";
import { ListItemNode, ListNode } from "@lexical/list";
import { HeadingNode, QuoteNode } from "@lexical/rich-text";

import LexicalEditorSurface from "./LexicalEditorSurface.vue";

const props = defineProps<{
  modelValue: string;
  lexicalValue?: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (event: "update:modelValue", value: string): void;
  (event: "update:lexicalValue", value: string): void;
}>();

const initialConfig = computed(() => ({
  namespace: "waken-wa-inspiration-editor",
  editable: true,
  nodes: [HeadingNode, QuoteNode, ListNode, ListItemNode, LinkNode],
  editorState: props.lexicalValue?.trim() ? props.lexicalValue : undefined,
  onError(error: Error) {
    console.error("Lexical editor error:", error);
  },
  theme: {
    paragraph: "lexical-paragraph",
    quote: "lexical-quote",
    heading: {
      h1: "lexical-heading lexical-heading-1",
      h2: "lexical-heading lexical-heading-2",
      h3: "lexical-heading lexical-heading-3",
    },
    list: {
      ul: "lexical-list lexical-list-ul",
      ol: "lexical-list lexical-list-ol",
      listitem: "lexical-list-item",
    },
    text: {
      bold: "lexical-text-bold",
      italic: "lexical-text-italic",
      underline: "lexical-text-underline",
      strikethrough: "lexical-text-strikethrough",
      code: "lexical-text-code",
    },
    link: "lexical-link",
  },
}));
</script>

<template>
  <LexicalComposer :initial-config="initialConfig">
    <LexicalEditorSurface
      :model-value="modelValue"
      :lexical-value="lexicalValue"
      :placeholder="placeholder"
      @update:model-value="emit('update:modelValue', $event)"
      @update:lexical-value="emit('update:lexicalValue', $event)"
    />
  </LexicalComposer>
</template>

