<script setup lang="ts">
import type { AppSection, SectionNavItem } from "@/app/types";

defineProps<{
  visibleSections: SectionNavItem[];
  activeSection: AppSection;
}>();

const emit = defineEmits<{
  select: [section: AppSection];
}>();
</script>

<template>
  <nav class="mobile-tabbar">
    <button
      v-for="section in visibleSections"
      :key="section.key"
      class="mobile-tab-item"
      :class="{ active: section.key === activeSection }"
      type="button"
      @click="emit('select', section.key)"
    >
      <i :class="section.icon" />
      <span>{{ section.title }}</span>
    </button>
  </nav>
</template>
