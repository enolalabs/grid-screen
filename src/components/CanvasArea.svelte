<script lang="ts">
  import CanvasToolbar from "./CanvasToolbar.svelte";
  import ScreenCanvas from "./ScreenCanvas.svelte";
  import ActionBar from "./ActionBar.svelte";

  interface Props {
    dragOverZone: number | null;
  }

  let { dragOverZone }: Props = $props();
</script>

<div class="canvas-area">
  <CanvasToolbar />
  <ScreenCanvas {dragOverZone} />
  <ActionBar />
</div>

<style>
  .canvas-area {
    display: flex; flex-direction: column;
    background: var(--bg); overflow: hidden;
  }
</style>
