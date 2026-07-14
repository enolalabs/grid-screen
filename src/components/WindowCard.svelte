<script lang="ts">
  import type { WindowDescriptor } from "$lib/shared-types";

  interface Props {
    window: WindowDescriptor;
    isAssigned: boolean;
    assignedZone: number | null;
    onDragStart: (windowId: string, x: number, y: number) => void;
  }

  let { window: win, isAssigned, assignedZone, onDragStart }: Props = $props();

  const shortName = $derived(win.app_name.slice(0, 2).toUpperCase());

  let dragging = $state(false);

  function handlePointerDown(e: PointerEvent) {
    if (isAssigned) return;
    dragging = true;
    (e.target as HTMLElement)?.setPointerCapture?.(e.pointerId);
    onDragStart(win.id, e.clientX, e.clientY);
  }
</script>

<li
  class="window-card"
  class:assigned={isAssigned}
  class:dragging={dragging}
  onpointerdown={handlePointerDown}
  role="listitem"
>
  <div class="app-icon" style="background:{win.icon_color}">{shortName}</div>
  <div class="window-info">
    <div class="window-app-name">{win.app_name}</div>
    <div class="window-title">{win.title}</div>
  </div>
  {#if isAssigned && assignedZone !== null}
    <span class="assign-badge">Zone {assignedZone + 1}</span>
  {/if}
</li>

<style>
  .window-card {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 12px; border-radius: var(--radius-sm);
    cursor: grab; transition: var(--transition);
    border: 1px solid transparent; margin-bottom: 4px; position: relative;
  }
  .window-card:hover { background: var(--surface-2); border-color: var(--border); }
  .window-card.assigned { opacity: 0.4; cursor: default; }
  .window-card.dragging { opacity: 0.3; border-color: var(--accent) !important; }
  .assign-badge {
    position: absolute; right: 10px; top: 50%; transform: translateY(-50%);
    font-size: 10px; font-weight: 600; padding: 2px 8px; border-radius: 4px;
    background: var(--accent-soft); color: var(--accent); pointer-events: none;
  }
  .app-icon {
    width: 28px; height: 28px; border-radius: 6px;
    display: flex; align-items: center; justify-content: center;
    font-size: 12px; font-weight: 700; flex-shrink: 0; color: #fff;
  }
  .window-info { flex: 1; min-width: 0; }
  .window-app-name {
    font-size: 13px; font-weight: 500;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .window-title {
    font-size: 11px; color: var(--text-mute);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
</style>
