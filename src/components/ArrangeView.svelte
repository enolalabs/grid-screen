<script lang="ts">
  import WindowCatalog from "./WindowCatalog.svelte";
  import CanvasArea from "./CanvasArea.svelte";
  import DetailPanel from "./DetailPanel.svelte";
  import { assignWindow } from "$lib/stores/assignments";

  let isDragging = $state(false);
  let draggedWindowId = $state<string | null>(null);
  let dragX = $state(0);
  let dragY = $state(0);
  let dragOverZone = $state<number | null>(null);

  function handleDragStart(windowId: string, x: number, y: number) {
    isDragging = true;
    draggedWindowId = windowId;
    dragX = x;
    dragY = y;
    dragOverZone = null;
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging) return;
    dragX = e.clientX;
    dragY = e.clientY;
    detectZoneUnderCursor(e.clientX, e.clientY);
  }

  function handlePointerUp(e: PointerEvent) {
    if (!isDragging) return;
    detectZoneUnderCursor(e.clientX, e.clientY);
    if (dragOverZone !== null && draggedWindowId !== null) {
      assignWindow(dragOverZone, draggedWindowId);
    }
    isDragging = false;
    draggedWindowId = null;
    dragOverZone = null;
  }

  function detectZoneUnderCursor(x: number, y: number) {
    const els = document.elementsFromPoint(x, y);
    const zoneEl = els.find((el) => {
      const attr = (el as HTMLElement).getAttribute?.("data-zone-index");
      return attr !== null && attr !== undefined;
    });
    if (zoneEl) {
      dragOverZone = Number(zoneEl.getAttribute("data-zone-index"));
    } else {
      dragOverZone = null;
    }
  }
</script>

<svelte:window
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
/>

<div class="arrange-main">
  <WindowCatalog onDragStart={handleDragStart} />
  <CanvasArea {dragOverZone} />
  <DetailPanel />
</div>

{#if isDragging && draggedWindowId}
  <div
    class="drag-ghost"
    style="left: {dragX + 12}px; top: {dragY - 20}px;"
  >
    Dragging...
  </div>
{/if}

<style>
  .arrange-main {
    display: grid;
    grid-template-columns: 280px 1fr 300px;
    flex: 1;
    overflow: hidden;
  }
  .drag-ghost {
    position: fixed; pointer-events: none; z-index: 1000; opacity: 0.8;
    background: var(--surface-2); border: 1px solid var(--accent);
    border-radius: var(--radius-sm); padding: 8px 14px; font-size: 13px;
    font-weight: 500; display: flex; align-items: center; gap: 8px;
  }
</style>
