<script setup lang="ts">
    const {text} = defineProps<{ text: string }>()
</script>
<template>
  <div class="text-secondary form-help">
    <Icon
      style="vertical-align: middle"
      name="ph:question"
      size="1.rem"
    />
    {{ text }}
  </div>
</template>

<style scoped>
  div.form-help {
    margin-top: .5rem;
  }
</style>