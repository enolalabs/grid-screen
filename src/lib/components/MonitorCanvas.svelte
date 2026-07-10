<script lang="ts">
  import type { Monitor, Zone } from "../types";

  interface Props {
    monitor: Monitor;
    draftZones: Zone[];
    selectedZoneId: string | null;
    onCreateZone: (monitorId: string, rect: { x: number; y: number; width: number; height: number }) => void;
    onSelectZone: (zoneId: string | null) => void;
    onChangeZone: (zoneId: string, patch: Partial<Zone>) => void;
    onDeleteZone: (zoneId: string) => void;
  }

  let {
    monitor,
    draftZones,
    selectedZoneId,
    onCreateZone,
    onSelectZone,
    onChangeZone,
    onDeleteZone,
  }: Props = $props();

  const GRID_COLS = 12;
  let canvasEl = $state<HTMLDivElement>();
  let canvasWidth = $state(0);
  let canvasHeight = $state(0);

  $effect(() => {
    if (!canvasEl) return;
    const maxW = Math.max(monitor.width, 1);
    const maxH = Math.max(monitor.height, 1);
    const scale = Math.min(600 / maxW, 300 / maxH, 1);
    canvasWidth = monitor.width * scale;
    canvasHeight = monitor.height * scale;
  });

  function snapToGrid(value: number): number {
    const step = 1.0 / GRID_COLS;
    return Math.round(value / step) * step;
  }

  function handlePointerDown(e: PointerEvent) {
    if (!canvasEl) return;
    if (e.target !== canvasEl && !(e.target as HTMLElement).classList.contains("grid-overlay")) return;
    const rect = canvasEl.getBoundingClientRect();
    const fx = (e.clientX - rect.left) / rect.width;
    const fy = (e.clientY - rect.top) / rect.height;
    const snappedX = snapToGrid(fx);
    const snappedY = snapToGrid(fy);
    const w = Math.min(4.0 / GRID_COLS, 1.0 - snappedX);
    const h = Math.min(4.0 / GRID_COLS, 1.0 - snappedY);
    onCreateZone(monitor.id, { x: snappedX, y: snappedY, width: w, height: h });
  }

  function handleZoneClick(e: MouseEvent, zoneId: string) {
    e.stopPropagation();
    onSelectZone(zoneId);
  }

  function handleZoneKeydown(e: KeyboardEvent, zone: Zone) {
    const step = e.shiftKey ? 0.01 : (1.0 / GRID_COLS);
    const patch: Partial<Zone> = {};
    let handled = false;

    if (e.ctrlKey || e.metaKey) {
      if (e.key === "ArrowRight") { patch.width = Math.min(zone.width + step, 1.0 - zone.x); handled = true; }
      if (e.key === "ArrowLeft") { patch.width = Math.max(zone.width - step, 1.0 / GRID_COLS); patch.x = Math.min(zone.x + step, zone.x); handled = true; }
      if (e.key === "ArrowDown") { patch.height = Math.min(zone.height + step, 1.0 - zone.y); handled = true; }
      if (e.key === "ArrowUp") { patch.height = Math.max(zone.height - step, 1.0 / GRID_COLS); patch.y = Math.min(zone.y + step, zone.y); handled = true; }
    } else {
      if (e.key === "ArrowRight") { patch.x = Math.min(zone.x + step, 1.0 - zone.width); handled = true; }
      if (e.key === "ArrowLeft") { patch.x = Math.max(zone.x - step, 0); handled = true; }
      if (e.key === "ArrowDown") { patch.y = Math.min(zone.y + step, 1.0 - zone.height); handled = true; }
      if (e.key === "ArrowUp") { patch.y = Math.max(zone.y - step, 0); handled = true; }
      if (e.key === "Delete") { onDeleteZone(zone.id); handled = true; }
    }

    if (handled) {
      e.preventDefault();
      onChangeZone(zone.id, patch);
    }
  }

  function gridLines(): number[] {
    return Array.from({ length: GRID_COLS + 1 }, (_, i) => i / GRID_COLS);
  }

  $effect(() => {
    if (!selectedZoneId && canvasEl) {
      const el = canvasEl.querySelector('[tabindex="0"]') as HTMLElement | null;
      el?.focus();
    }
  });
</script>

<div
  class="monitor-canvas"
  bind:this={canvasEl}
  style="width: {canvasWidth}px; height: {canvasHeight}px;"
  onpointerdown={handlePointerDown}
  role="application"
  aria-label="Monitor {monitor.name} zone editor"
>
  <svg class="grid-overlay" width={canvasWidth} height={canvasHeight}>
    {#each gridLines() as fx}
      <line x1={fx * canvasWidth} y1="0" x2={fx * canvasWidth} y2={canvasHeight} class="grid-line" />
    {/each}
    {#each gridLines() as fy}
      <line x1="0" y1={fy * canvasHeight} x2={canvasWidth} y2={fy * canvasHeight} class="grid-line" />
    {/each}
  </svg>

  {#each draftZones as zone (zone.id)}
    {@const isSelected = zone.id === selectedZoneId}
    <div
      class="zone"
      class:selected={isSelected}
      style="
        left: {zone.x * canvasWidth}px;
        top: {zone.y * canvasHeight}px;
        width: {zone.width * canvasWidth}px;
        height: {zone.height * canvasHeight}px;
      "
      tabindex="0"
      role="button"
      aria-label={zone.name}
      aria-pressed={isSelected}
      onclick={(e: MouseEvent) => handleZoneClick(e, zone.id)}
      onkeydown={(e: KeyboardEvent) => handleZoneKeydown(e, zone)}
    >
      <span class="zone-label">{zone.name}</span>
      <span class="zone-coords">
        x{zone.x.toFixed(3)} y{zone.y.toFixed(3)} w{zone.width.toFixed(3)} h{zone.height.toFixed(3)}
      </span>
    </div>
  {/each}
</div>

<style>
  .monitor-canvas {
    position: relative;
    background: var(--canvas);
    border: 2px solid var(--border);
    border-radius: var(--radius-panel);
    cursor: crosshair;
    overflow: hidden;
  }

  .grid-overlay {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .grid-line {
    stroke: var(--surface-3);
    stroke-width: 0.5;
  }

  .zone {
    position: absolute;
    border: 2px dashed rgba(139, 92, 246, 0.5);
    background: rgba(139, 92, 246, 0.08);
    border-radius: var(--radius-control);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    min-width: 40px;
    min-height: 28px;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .zone:hover {
    border-color: rgba(139, 92, 246, 0.8);
    background: rgba(139, 92, 246, 0.15);
  }

  .zone.selected {
    border: 2px solid var(--primary);
    background: rgba(139, 92, 246, 0.2);
  }

  .zone:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  @media (prefers-reduced-motion: reduce) {
    .zone { transition: none; }
  }

  .zone-label {
    font-size: 11px;
    font-family: var(--sans);
    color: var(--text);
    font-weight: 500;
    pointer-events: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 90%;
  }

  .zone-coords {
    font-family: var(--mono);
    font-size: 9px;
    color: var(--text-muted);
    pointer-events: none;
    white-space: nowrap;
  }
</style>
