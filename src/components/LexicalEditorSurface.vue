<script setup lang="ts">
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { $toggleLink } from "@lexical/link";
import {
  INSERT_ORDERED_LIST_COMMAND,
  INSERT_UNORDERED_LIST_COMMAND,
} from "@lexical/list";
import { $createHeadingNode, $createQuoteNode } from "@lexical/rich-text";
import { $setBlocksType } from "@lexical/selection";
import {
  $createParagraphNode,
  $createTextNode,
  $getRoot,
  $getSelection,
  $isRangeSelection,
  CAN_REDO_COMMAND,
  CAN_UNDO_COMMAND,
  COMMAND_PRIORITY_LOW,
  FORMAT_TEXT_COMMAND,
  REDO_COMMAND,
  UNDO_COMMAND,
  type EditorState,
} from "lexical";
import { useLexicalComposer } from "lexical-vue/LexicalComposer";
import { ContentEditable } from "lexical-vue/LexicalContentEditable";
import { HistoryPlugin } from "lexical-vue/LexicalHistoryPlugin";
import { LinkPlugin } from "lexical-vue/LexicalLinkPlugin";
import { ListPlugin } from "lexical-vue/LexicalListPlugin";
import { OnChangePlugin } from "lexical-vue/LexicalOnChangePlugin";
import { RichTextPlugin } from "lexical-vue/LexicalRichTextPlugin";

import { lexicalTextContent } from "../lib/inspirationRichText";

const { t } = useI18n();

const props = defineProps<{
  modelValue: string;
  lexicalValue?: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (event: "update:modelValue", value: string): void;
  (event: "update:lexicalValue", value: string): void;
}>();

const editor = useLexicalComposer();
const canUndo = ref(false);
const canRedo = ref(false);
const linkDraft = ref("");
const showLinkInput = ref(false);
const isEmpty = computed(() => !props.modelValue.trim());

let unregister: (() => void) | null = null;
let lastSerialized = "";
let lastPlainText = "";
let applyingExternalState = false;

function buildTextState(value: string) {
  return () => {
    const root = $getRoot();
    root.clear();

    const blocks = value
      .replace(/\r\n/g, "\n")
      .split(/\n{2,}/)
      .map((block) => block.trim())
      .filter(Boolean);

    if (blocks.length === 0) {
      root.append($createParagraphNode());
      return;
    }

    for (const block of blocks) {
      const paragraph = $createParagraphNode();
      const lines = block.split("\n");
      lines.forEach((line, index) => {
        paragraph.append($createTextNode(line));
        if (index < lines.length - 1) {
          paragraph.append($createTextNode("\n"));
        }
      });
      root.append(paragraph);
    }
  };
}

function applyEditorState(serialized: string) {
  applyingExternalState = true;
  try {
    const nextState = editor.parseEditorState(serialized);
    editor.setEditorState(nextState);
    lastSerialized = serialized;
    lastPlainText = lexicalTextContent(serialized);
  } finally {
    applyingExternalState = false;
  }
}

function applyPlainTextState(value: string) {
  applyingExternalState = true;
  try {
    editor.update(buildTextState(value));
    lastSerialized = "";
    lastPlainText = value;
  } finally {
    applyingExternalState = false;
  }
}

function emitEditorState(editorState: EditorState) {
  const serialized = JSON.stringify(editorState.toJSON());
  const plainText = lexicalTextContent(serialized);

  if (serialized === lastSerialized && plainText === lastPlainText) {
    return;
  }

  lastSerialized = serialized;
  lastPlainText = plainText;
  emit("update:lexicalValue", serialized);
  emit("update:modelValue", plainText);
}

function handleOnChange(payload: Event): void;
function handleOnChange(editorState: EditorState): void;
function handleOnChange(payload: Event | EditorState) {
  if (!(payload as EditorState).read) {
    return;
  }

  const editorState = payload as EditorState;
  if (applyingExternalState) return;
  emitEditorState(editorState);
}

function setBlock(type: "paragraph" | "heading" | "quote") {
  editor.update(() => {
    const selection = $getSelection();
    if (!$isRangeSelection(selection)) return;

    if (type === "paragraph") {
      $setBlocksType(selection, () => $createParagraphNode());
      return;
    }

    if (type === "heading") {
      $setBlocksType(selection, () => $createHeadingNode("h2"));
      return;
    }

    $setBlocksType(selection, () => $createQuoteNode());
  });
}

function insertLink() {
  const url = linkDraft.value.trim();
  if (!url) {
    showLinkInput.value = false;
    return;
  }
  editor.update(() => {
    $toggleLink(url);
  });
  linkDraft.value = "";
  showLinkInput.value = false;
}

function dispatchUndo() {
  editor.dispatchCommand(UNDO_COMMAND, undefined);
}

function dispatchRedo() {
  editor.dispatchCommand(REDO_COMMAND, undefined);
}

function formatText(type: "bold" | "italic" | "underline" | "code") {
  editor.dispatchCommand(FORMAT_TEXT_COMMAND, type);
}

function insertList(type: "bullet" | "number") {
  if (type === "bullet") {
    editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined);
    return;
  }
  editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined);
}

function focusEditor() {
  editor.focus();
}

unregister = [
  editor.registerCommand(
    CAN_UNDO_COMMAND,
    (payload) => {
      canUndo.value = Boolean(payload);
      return false;
    },
    COMMAND_PRIORITY_LOW,
  ),
  editor.registerCommand(
    CAN_REDO_COMMAND,
    (payload) => {
      canRedo.value = Boolean(payload);
      return false;
    },
    COMMAND_PRIORITY_LOW,
  ),
].reduceRight<() => void>(
  (dispose, current) => () => {
    current();
    dispose();
  },
  () => {},
);

const initialLexical = props.lexicalValue?.trim() ?? "";
if (initialLexical) {
  applyEditorState(initialLexical);
} else {
  applyPlainTextState(props.modelValue);
}

watch(
  () => props.lexicalValue,
  (nextValue) => {
    const normalized = nextValue?.trim() ?? "";
    if (normalized && normalized !== lastSerialized) {
      applyEditorState(normalized);
      return;
    }

    if (!normalized && !applyingExternalState && props.modelValue !== lastPlainText) {
      applyPlainTextState(props.modelValue);
    }
  },
);

watch(
  () => props.modelValue,
  (nextValue) => {
    const hasLexical = Boolean(props.lexicalValue?.trim());
    if (hasLexical || applyingExternalState || nextValue === lastPlainText) {
      return;
    }
    applyPlainTextState(nextValue);
  },
);

onBeforeUnmount(() => {
  unregister?.();
});
</script>

<template>
  <div class="lexical-editor-shell">
    <div class="lexical-toolbar">
      <Button :label="t('lexicalEditor.actions.undo')" text size="small" :disabled="!canUndo" @click="dispatchUndo" />
      <Button :label="t('lexicalEditor.actions.redo')" text size="small" :disabled="!canRedo" @click="dispatchRedo" />
      <span class="lexical-divider" />
      <Button :label="t('lexicalEditor.actions.paragraph')" text size="small" @click="setBlock('paragraph')" />
      <Button :label="t('lexicalEditor.actions.heading')" text size="small" @click="setBlock('heading')" />
      <Button :label="t('lexicalEditor.actions.quote')" text size="small" @click="setBlock('quote')" />
      <Button :label="t('lexicalEditor.actions.bold')" text size="small" @click="formatText('bold')" />
      <Button :label="t('lexicalEditor.actions.italic')" text size="small" @click="formatText('italic')" />
      <Button :label="t('lexicalEditor.actions.underline')" text size="small" @click="formatText('underline')" />
      <Button :label="t('lexicalEditor.actions.code')" text size="small" @click="formatText('code')" />
      <Button :label="t('lexicalEditor.actions.bulletList')" text size="small" @click="insertList('bullet')" />
      <Button :label="t('lexicalEditor.actions.orderedList')" text size="small" @click="insertList('number')" />
      <Button
        :label="t('lexicalEditor.actions.link')"
        text
        size="small"
        @click="showLinkInput = !showLinkInput"
      />
    </div>

    <div v-if="showLinkInput" class="lexical-link-row">
      <InputText
        v-model="linkDraft"
        :placeholder="t('lexicalEditor.link.placeholder')"
        @keydown.enter.prevent="insertLink"
      />
      <Button :label="t('lexicalEditor.actions.applyLink')" size="small" @click="insertLink" />
    </div>

    <div class="lexical-editor-frame" @click="focusEditor">
      <RichTextPlugin>
        <template #contentEditable>
          <ContentEditable class="lexical-editor-root">
            <template #placeholder>
              <div v-if="isEmpty" class="lexical-placeholder">
                {{ placeholder || t("lexicalEditor.placeholder.default") }}
              </div>
            </template>
          </ContentEditable>
        </template>
      </RichTextPlugin>
      <HistoryPlugin :delay="300" />
      <ListPlugin />
      <LinkPlugin />
      <OnChangePlugin @change="handleOnChange" />
    </div>
  </div>
</template>
