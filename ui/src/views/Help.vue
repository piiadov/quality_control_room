<script setup>
import { computed } from 'vue';
import { marked } from 'marked';
import helpMarkdown from 'virtual:help-markdown';

const htmlContent = computed(() => marked(helpMarkdown));
</script>

<template>
  <div class="p-8 max-w-4xl mx-auto">
    <div class="markdown-content" v-html="htmlContent"></div>
  </div>
</template>

<style scoped>
.markdown-content {
  color: var(--text-color);
  line-height: 1.6;
}

.markdown-content :deep(h1) {
  font-size: 2rem;
  font-weight: 700;
  margin-bottom: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-color);
}

.markdown-content :deep(h2) {
  font-size: 1.5rem;
  font-weight: 600;
  margin-top: 2rem;
  margin-bottom: 0.75rem;
}

.markdown-content :deep(h3) {
  font-size: 1.25rem;
  font-weight: 600;
  margin-top: 1.5rem;
  margin-bottom: 0.5rem;
}

.markdown-content :deep(p) {
  margin-bottom: 1rem;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin-left: 1.5rem;
  margin-bottom: 1rem;
}

.markdown-content :deep(li) {
  margin-bottom: 0.25rem;
}

.markdown-content :deep(ul) {
  list-style-type: disc;
}

.markdown-content :deep(ol) {
  list-style-type: decimal;
}

.markdown-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 1rem;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid var(--border-color);
  padding: 0.5rem 1rem;
  text-align: left;
}

.markdown-content :deep(th) {
  background-color: var(--background-secondary);
  font-weight: 600;
}

.markdown-content :deep(code) {
  background-color: var(--background-secondary);
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-family: monospace;
}

.markdown-content :deep(pre) {
  background-color: var(--background-secondary);
  padding: 1rem;
  border-radius: 0.5rem;
  overflow-x: auto;
  margin-bottom: 1rem;
}

.markdown-content :deep(blockquote) {
  border-left: 4px solid var(--border-color);
  padding-left: 1rem;
  margin-left: 0;
  color: var(--text-secondary);
  font-style: italic;
}

.markdown-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 2rem 0;
}

.markdown-content :deep(strong) {
  font-weight: 600;
}

.markdown-content :deep(a) {
  color: var(--link-color, #3b82f6);
  text-decoration: underline;
}

.markdown-content :deep(a:hover) {
  opacity: 0.8;
}
</style>
