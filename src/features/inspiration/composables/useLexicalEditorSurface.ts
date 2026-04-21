import { computed, onBeforeUnmount, ref, watch } from "vue";
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

import { lexicalTextContent } from "@/lib/inspirationRichText";

interface LexicalEditorSurfaceProps {
  modelValue: string;
  lexicalValue?: string;
}

interface LexicalEditorSurfaceEmit {
  (event: "update:modelValue", value: string): void;
  (event: "update:lexicalValue", value: string): void;
}

export function useLexicalEditorSurface(
  props: LexicalEditorSurfaceProps,
  emit: LexicalEditorSurfaceEmit,
) {
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

  function applyLink() {
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

  return {
    applyLink,
    canRedo,
    canUndo,
    dispatchRedo,
    dispatchUndo,
    focusEditor,
    formatText,
    handleOnChange,
    insertList,
    isEmpty,
    linkDraft,
    setBlock,
    showLinkInput,
  };
}
