<script lang="ts">
  import type { WindowDescriptor } from "$lib/shared-types";
  import { removeWindowFromZone } from "$lib/stores/assignments";

  interface Props {
    zoneIndex: number;
    assignedWindow: WindowDescriptor | null;
    isDragOver: boolean;
  }

  let { zoneIndex, assignedWindow, isDragOver }: Props = $props();
</script>

<div
  class="zone"
  class:has-window={assignedWindow !== null}
  class:drag-over={isDragOver}
  data-zone-index={zoneIndex}
>
  {#if assignedWindow}
    <button class="zone-remove" onclick={() => removeWindowFromZone(zoneIndex)}>
      x
    </button>
    <span class="zone-label">Zone {zoneIndex + 1}</span>
    <div class="zone-window">
      <div
        class="app-icon"
        style="width:20px;height:20px;font-size:10px;background:{assignedWindow.icon_color}"
      >
        {assignedWindow.app_name.slice(0, 2).toUpperCase()}
      </div>
      {assignedWindow.app_name}
    </div>
  {:else}
    <span class="zone-label">Zone {zoneIndex + 1}</span>
    <span class="zone-hint">Drop window here</span>
  {/if}
</div>

<style>
  .zone {
    border: 2px dashed var(--border); border-radius: var(--radius);
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; cursor: pointer; transition: var(--transition);
    position: relative; min-height: 60px;
  }
  .zone:hover { border-color: var(--accent); background: var(--accent-soft); }
  .zone.drag-over {
    border-color: var(--accent); background: var(--accent-soft);
    transform: scale(1.02);
  }
  .zone.has-window {
    border-style: solid; border-color: var(--accent);
    background: linear-gradient(135deg, var(--accent-soft), transparent);
  }
  .zone-label {
    font-size: 11px; font-weight: 600; color: var(--text-mute);
    text-transform: uppercase; letter-spacing: 0.05em;
  }
  .zone.has-window .zone-label { color: var(--accent); }
  .zone-hint { font-size: 11px; color: var(--text-mute); margin-top: 6px; }
  .zone-window {
    display: flex; align-items: center; gap: 8px;
    margin-top: 8px; padding: 6px 12px; background: var(--surface-3);
    border-radius: var(--radius-sm); font-size: 12px; font-weight: 500;
  }
  .zone-remove {
    position: absolute; top: 6px; right: 6px;
    width: 20px; height: 20px; border-radius: 50%;
    background: var(--surface-3); border: none; cursor: pointer;
    display: none; align-items: center; justify-content: center;
    font-size: 12px; color: var(--text-dim); transition: var(--transition);
  }
  .zone.has-window .zone-remove { display: flex; }
  .zone-remove:hover { background: var(--danger); color: #fff; }
  .app-icon {
    border-radius: 6px; display: flex; align-items: center;
    justify-content: center; font-weight: 700; flex-shrink: 0; color: #fff;
  }
</style>
