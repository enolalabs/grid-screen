<script lang="ts">
  import LayoutSliders from "./LayoutSliders.svelte";
  import SnapControls from "./SnapControls.svelte";
  import SystemStatusPanel from "./SystemStatusPanel.svelte";
</script>

<div class="detail-panel">
  <LayoutSliders />
  <SnapControls />
  <SystemStatusPanel />
</div>

<style>
  .detail-panel {
    background: var(--surface); border-left: 1px solid var(--border);
    display: flex; flex-direction: column; overflow-y: auto;
  }
</style>
