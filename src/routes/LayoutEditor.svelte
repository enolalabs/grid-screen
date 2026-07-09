<script lang="ts">
  import { onMount } from "svelte";
  import { currentState } from "../lib/stores";
  import { applyLayout, saveLayout } from "../lib/ipc";
  import type { Monitor, Zone, Layout } from "../lib/types";

  let monitors = $state<Monitor[]>([]);
  let zones = $state<Map<string, Zone[]>>(new Map());
  const GRID_COLS = 12;
  let saveName = $state("");
  let deleteTarget = $state<{ zone: Zone, monitorId: string } | null>(null);
  let toastMessage = $state("");
  let toastType = $state<"info" | "error">("info");

  onMount(() => {
    const unsub = currentState.subscribe(s => {
      if (s) {
        monitors = s.monitors;
        for (const layout of s.active_layouts) {
          zones.set(layout.monitor_id, layout.zones);
        }
      }
    });
    return unsub;
  });

  function notify(message: string, type: "info" | "error" = "info") {
    toastMessage = message;
    toastType = type;
    setTimeout(() => toastMessage = "", 4000);
  }

  function handleCreateZone(monitorId: string, x: number, y: number, w: number, h: number) {
    const monitorZones = zones.get(monitorId) ?? [];
    const colW = 1.0 / GRID_COLS;
    const rowH = 1.0 / GRID_COLS;
    const snappedX = Math.round(x / colW) * colW;
    const snappedY = Math.round(y / rowH) * rowH;
    const snappedW = Math.max(Math.round(w / colW) * colW, colW);
    const snappedH = Math.max(Math.round(h / rowH) * rowH, rowH);

    const zone: Zone = {
      id: crypto.randomUUID(),
      name: `Zone ${monitorZones.length + 1}`,
      x: snappedX, y: snappedY,
      width: Math.min(snappedW, 1.0 - snappedX),
      height: Math.min(snappedH, 1.0 - snappedY),
      gap: 4, margin: 8,
    };
    zones.set(monitorId, [...monitorZones, zone]);
  }

  function handleDeleteZone(monitorId: string, zoneId: string) {
    const monitorZones = (zones.get(monitorId) ?? []).filter(z => z.id !== zoneId);
    zones.set(monitorId, monitorZones);
  }

  async function handleApply() {
    try {
      for (const [monitorId, zs] of zones) {
        await applyLayout({ zones: zs, monitor_id: monitorId });
      }
    } catch (e) {
      notify(`Failed to apply layout: ${e}`, "error");
    }
  }

  async function handleSave() {
    if (!saveName.trim()) return;
    try {
      for (const [monitorId, zs] of zones) {
        await saveLayout(saveName, zs, monitorId);
      }
      saveName = "";
    } catch (e) {
      notify(`Failed to save: ${e}`, "error");
    }
  }

  function getMonitorStyle(m: Monitor) {
    const maxW = Math.max(...monitors.map(x => x.width), 1);
    const maxH = Math.max(...monitors.map(x => x.height), 1);
    const scale = Math.min(600 / maxW, 300 / maxH, 1);
    return `width: ${m.width * scale}px; height: ${m.height * scale}px;`;
  }

  function zoneStyle(z: Zone, m: Monitor) {
    const maxW = Math.max(...monitors.map(x => x.width), 1);
    const maxH = Math.max(...monitors.map(x => x.height), 1);
    const scale = Math.min(600 / maxW, 300 / maxH, 1);
    return `
      left: ${z.x * m.width * scale}px;
      top: ${z.y * m.height * scale}px;
      width: ${z.width * m.width * scale}px;
      height: ${z.height * m.height * scale}px;
    `;
  }
</script>

<div class="editor">
  <div class="toolbar">
    <input bind:value={saveName} placeholder="Layout name..." />
    <button onclick={handleSave} disabled={!saveName.trim()}>Save</button>
    <button onclick={handleApply}>Apply Live</button>
  </div>

  {#if monitors.length === 0}
    <p class="empty">No monitors detected. Connect a display to begin.</p>
  {:else}
    <div class="monitors">
      {#each monitors as monitor (monitor.id)}
        <div class="monitor-panel">
          <div class="monitor-label">{monitor.name} ({monitor.width}&times;{monitor.height})</div>
          <div
            class="monitor-canvas"
            style={getMonitorStyle(monitor)}
            onpointerdown={(e) => {
              if (e.target === e.currentTarget) {
                const rect = e.currentTarget.getBoundingClientRect();
                const x = (e.clientX - rect.left) / rect.width;
                const y = (e.clientY - rect.top) / rect.height;
                handleCreateZone(monitor.id, x, y, 0.3, 0.3);
              }
            }}
            role="application"
            aria-label="Monitor {monitor.name} zone editor"
          >
            {#each zones.get(monitor.id) ?? [] as zone (zone.id)}
              <div
                class="zone"
                style={zoneStyle(zone, monitor)}
                tabindex="0"
                role="region"
                aria-label="{zone.name} — drag to resize, double-click to rename"
                ondblclick={() => {
                  const name = prompt("Zone name:", zone.name);
                  if (name) zone.name = name.trim().slice(0, 64);
                  zones = new Map(zones);
                }}
                oncontextmenu={(e) => {
                  e.preventDefault();
                  deleteTarget = { zone, monitorId: monitor.id };
                }}
                onkeydown={(e) => {
                  const step = e.shiftKey ? 0.01 : (1.0 / GRID_COLS);
                  if (e.key === "ArrowRight") zone.x = Math.min(zone.x + step, 1.0 - zone.width);
                  if (e.key === "ArrowLeft") zone.x = Math.max(zone.x - step, 0);
                  if (e.key === "ArrowDown") zone.y = Math.min(zone.y + step, 1.0 - zone.height);
                  if (e.key === "ArrowUp") zone.y = Math.max(zone.y - step, 0);
                  if (e.key === "Delete") { deleteTarget = { zone, monitorId: monitor.id }; }
                  zones = new Map(zones);
                }}
              >
                <span class="zone-label">{zone.name}</span>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if deleteTarget}
  <div class="confirm-overlay" role="alertdialog" aria-label="Delete zone">
    <div class="confirm-card">
      <p>Delete zone "{deleteTarget.zone.name}"?</p>
      <div class="confirm-actions">
        <button onclick={() => { handleDeleteZone(deleteTarget.monitorId, deleteTarget.zone.id); deleteTarget = null; }}>Delete</button>
        <button onclick={() => deleteTarget = null}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

{#if toastMessage}
  <div class="toast toast-{toastType}" role="status">{toastMessage}</div>
{/if}

<style>
  .editor { display: flex; flex-direction: column; gap: 12px; }
  .toolbar { display: flex; gap: 8px; align-items: center; }
  .toolbar input { padding: 6px 10px; background: #313244; border: 1px solid #45475a; color: #cdd6f4; border-radius: 4px; }
  .toolbar button { padding: 6px 16px; background: #7C3AED; color: white; border: none; border-radius: 4px; cursor: pointer; }
  .toolbar button:disabled { opacity: 0.5; cursor: default; }
  .empty { color: #6c7086; padding: 24px; }
  .monitors { display: flex; flex-wrap: wrap; gap: 24px; }
  .monitor-panel { display: flex; flex-direction: column; gap: 4px; }
  .monitor-label { font-size: 12px; color: #a6adc8; }
  .monitor-canvas { position: relative; background: #11111b; border: 2px solid #45475a; border-radius: 4px; cursor: crosshair; }
  .zone { position: absolute; border: 2px solid #7C3AED; background: rgba(124, 58, 237, 0.15); border-radius: 4px; display: flex; align-items: center; justify-content: center; cursor: move; min-width: 40px; min-height: 24px; }
  .zone:focus { outline: 2px solid white; outline-offset: 2px; }
  .zone-label { font-size: 11px; color: #cdd6f4; pointer-events: none; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .confirm-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .confirm-card { background: #1e1e2e; padding: 20px 28px; border-radius: 10px; }
  .confirm-card p { margin: 0 0 16px; }
  .confirm-actions { display: flex; gap: 10px; justify-content: flex-end; }
  .confirm-actions button { padding: 6px 16px; border: none; border-radius: 4px; cursor: pointer; }
  .confirm-actions button:first-child { background: #f38ba8; color: #1e1e2e; }
  .confirm-actions button:last-child { background: #45475a; color: #cdd6f4; }
  .toast { position: fixed; bottom: 16px; right: 16px; padding: 10px 20px; border-radius: 6px; font-size: 14px; z-index: 200; animation: slideIn 0.3s ease; }
  .toast-info { background: #313244; color: #cdd6f4; }
  .toast-error { background: #f38ba8; color: #1e1e2e; }
  @keyframes slideIn { from { transform: translateX(100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
</style>
