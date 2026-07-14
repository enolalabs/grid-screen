<script lang="ts">
  import { selectedLayout } from "$lib/stores/layout";
  import { screens, selectedScreenId } from "$lib/stores/screen";
  import { windows } from "$lib/stores/windows";
  import { assignments } from "$lib/stores/assignments";
  import ZoneSlot from "./ZoneSlot.svelte";

  interface Props {
    dragOverZone: number | null;
  }

  let { dragOverZone }: Props = $props();

  const currentScreen = $derived(
    $screens.find((s) => s.id === $selectedScreenId)
  );

  function getAssignedWindow(zoneIndex: number) {
    const windowId = $assignments[zoneIndex];
    if (!windowId) return null;
    return $windows.find((w) => w.id === windowId) ?? null;
  }

  function gridStyle(layout: NonNullable<typeof $selectedLayout>): string {
    if (!layout) return "";
    const ratio = layout.ratio ?? null;
    let cols = layout.columns;
    if (layout.zones === 2 && ratio !== null && ratio !== 50) {
      const rest = 100 - ratio;
      cols = `${ratio}fr ${rest}fr`;
    }
    return `grid-template-columns: ${cols}; grid-template-rows: ${layout.rows ?? "1fr"}; gap: ${layout.gap_px}px; padding: ${layout.margin_px}px;`;
  }
</script>

<div class="canvas-container">
  <div class="screen-mock" style={$selectedLayout ? gridStyle($selectedLayout) : ""}>
    {#if currentScreen}
      <div class="screen-label-bar">
        {currentScreen.id} &nbsp; {currentScreen.resolution}
      </div>
    {/if}
    {#if $selectedLayout}
      {#each Array.from({ length: $selectedLayout.zones }, (_, i) => i) as i (i)}
        <div
          style={$selectedLayout.span_first && i === 0 ? "grid-row: span 2;" : ""}
        >
          <ZoneSlot
            zoneIndex={i}
            assignedWindow={getAssignedWindow(i)}
            isDragOver={dragOverZone === i}
          />
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .canvas-container {
    flex: 1; display: flex; align-items: center; justify-content: center;
    padding: 32px; overflow: hidden;
  }
  .screen-mock {
    width: 100%; max-width: 720px; aspect-ratio: 16/10;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 12px; display: grid; position: relative; overflow: hidden;
  }
  .screen-label-bar {
    position: absolute; top: -24px; left: 0;
    font-size: 11px; color: var(--text-mute); font-weight: 500;
  }
</style>
