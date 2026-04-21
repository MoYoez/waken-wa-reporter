<script setup lang="ts">
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { useI18n } from "vue-i18n";

const linkDraft = defineModel<string>("linkDraft", { required: true });
const showLinkInput = defineModel<boolean>("showLinkInput", { required: true });

defineProps<{
  canUndo: boolean;
  canRedo: boolean;
}>();

defineEmits<{
  undo: [];
  redo: [];
  setBlock: [type: "paragraph" | "heading" | "quote"];
  formatText: [type: "bold" | "italic" | "underline" | "code"];
  insertList: [type: "bullet" | "number"];
  applyLink: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="lexical-toolbar">
    <Button :label="t('lexicalEditor.actions.undo')" text size="small" :disabled="!canUndo" @click="$emit('undo')" />
    <Button :label="t('lexicalEditor.actions.redo')" text size="small" :disabled="!canRedo" @click="$emit('redo')" />
    <span class="lexical-divider" />
    <Button :label="t('lexicalEditor.actions.paragraph')" text size="small" @click="$emit('setBlock', 'paragraph')" />
    <Button :label="t('lexicalEditor.actions.heading')" text size="small" @click="$emit('setBlock', 'heading')" />
    <Button :label="t('lexicalEditor.actions.quote')" text size="small" @click="$emit('setBlock', 'quote')" />
    <Button :label="t('lexicalEditor.actions.bold')" text size="small" @click="$emit('formatText', 'bold')" />
    <Button :label="t('lexicalEditor.actions.italic')" text size="small" @click="$emit('formatText', 'italic')" />
    <Button :label="t('lexicalEditor.actions.underline')" text size="small" @click="$emit('formatText', 'underline')" />
    <Button :label="t('lexicalEditor.actions.code')" text size="small" @click="$emit('formatText', 'code')" />
    <Button :label="t('lexicalEditor.actions.bulletList')" text size="small" @click="$emit('insertList', 'bullet')" />
    <Button :label="t('lexicalEditor.actions.orderedList')" text size="small" @click="$emit('insertList', 'number')" />
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
      @keydown.enter.prevent="$emit('applyLink')"
    />
    <Button :label="t('lexicalEditor.actions.applyLink')" size="small" @click="$emit('applyLink')" />
  </div>
</template>
