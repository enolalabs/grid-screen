<script lang="ts">
  import type { Zone } from "../types";

  interface Props {
    layoutId: string;
    zones: Zone[];
    label: string;
  }

  let { layoutId, zones, label }: Props = $props();

  function clamp(n: number): number {
    if (!isFinite(n)) return 0;
    return Math.max(0, Math.min(1, n));
  }

  const zoneKey = $derived(String(zones.map(z => `${z.x},${z.y},${z.width},${z.height}`).join("|")));
  const memoKey = $derived(`${layoutId}:${zoneKey}`);

  const clampedZones = $derived(zones.map(z => ({
    ...z,
    x: clamp(z.x),
    y: clamp(z.y),
    width: Math.max(1 / 12, clamp(z.width)),
    height: Math.max(1 / 12, clamp(z.height)),
  })));
</script>

<div class="thumbnail" data-memo={memoKey} aria-label={label}>
  <div class="thumbnail-panel">
    {#each clampedZones as zone (zone.id)}
      <div
        class="thumbnail-zone"
        style="left: {zone.x * 100}%; top: {zone.y * 100}%; width: {zone.width * 100}%; height: {zone.height * 100}%;"
      ></div>
    {/each}
  </div>
  <span class="thumbnail-label">{label}</span>
</div>

<style>
  .thumbnail {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .thumbnail-panel {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 10;
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    overflow: hidden;
  }

  .thumbnail-zone {
    position: absolute;
    background: var(--surface-3);
    border: 1px dashed var(--text-muted);
    opacity: 0.8;
  }

  .thumbnail-label {
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
</style>
