<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { ContentEditable } from "lexical-vue/LexicalContentEditable";
import { HistoryPlugin } from "lexical-vue/LexicalHistoryPlugin";
import { LinkPlugin } from "lexical-vue/LexicalLinkPlugin";
import { ListPlugin } from "lexical-vue/LexicalListPlugin";
import { OnChangePlugin } from "lexical-vue/LexicalOnChangePlugin";
import { RichTextPlugin } from "lexical-vue/LexicalRichTextPlugin";

import LexicalEditorToolbar from "@/features/inspiration/components/LexicalEditorToolbar.vue";
import { useLexicalEditorSurface } from "@/features/inspiration/composables/useLexicalEditorSurface";

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

const {
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
} = useLexicalEditorSurface(props, emit);
</script>

<template>
  <div class="lexical-editor-shell">
    <LexicalEditorToolbar
      v-model:link-draft="linkDraft"
      v-model:show-link-input="showLinkInput"
      :can-undo="canUndo"
      :can-redo="canRedo"
      @undo="dispatchUndo"
      @redo="dispatchRedo"
      @set-block="setBlock"
      @format-text="formatText"
      @insert-list="insertList"
      @apply-link="applyLink"
    />

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

