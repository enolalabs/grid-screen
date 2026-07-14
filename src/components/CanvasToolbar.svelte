<script lang="ts">
  import ScreenSelector from "./ScreenSelector.svelte";
  import LayoutSelector from "./LayoutSelector.svelte";
  import LayoutQuickButtons from "./LayoutQuickButtons.svelte";
</script>

<div class="canvas-toolbar">
  <ScreenSelector />
  <LayoutSelector />
  <LayoutQuickButtons />
</div>

<style>
  .canvas-toolbar {
    display: flex; align-items: center; gap: 12px;
    padding: 12px 20px; border-bottom: 1px solid var(--border);
  }
</style>
