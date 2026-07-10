<script lang="ts">
  import { onMount } from "svelte";
  import { currentState, savedLayouts } from "../lib/stores";
  import { applyLayout, saveLayout, getCurrentState } from "../lib/ipc";
  import { notify } from "../lib/notifications";
  import type { Monitor, Zone, Layout } from "../lib/types";
  import Panel from "../lib/components/Panel.svelte";
  import Button from "../lib/components/Button.svelte";
  import EmptyState from "../lib/components/EmptyState.svelte";
  import MonitorCanvas from "../lib/components/MonitorCanvas.svelte";
  import ZoneInspector from "../lib/components/ZoneInspector.svelte";
  import Badge from "../lib/components/Badge.svelte";

  let monitors = $state<Monitor[]>([]);
  let draftZones = $state<Map<string, Zone[]>>(new Map());
  let selectedZoneId = $state<string | null>(null);
  let selectedMonitorId = $state<string | null>(null);
  let saveName = $state("");
  let isApplying = $state(false);
  let isSaving = $state(false);
  let errorMessage = $state("");
  const GRID_COLS = 12;

  onMount(() => {
    const unsub = currentState.subscribe(s => {
      if (s) {
        monitors = s.monitors;
        const map = new Map<string, Zone[]>();
        for (const layout of s.active_layouts) {
          map.set(layout.monitor_id, [...layout.zones]);
        }
        draftZones = map;
      }
    });
    return unsub;
  });

  function handleCreateZone(monitorId: string, rect: { x: number; y: number; width: number; height: number }) {
    const existing = draftZones.get(monitorId) ?? [];
    const id = crypto.randomUUID();
    const zone: Zone = {
      id,
      name: `Zone ${existing.length + 1}`,
      x: rect.x,
      y: rect.y,
      width: rect.width,
      height: rect.height,
      gap: 4,
      margin: 8,
    };
    const updated = new Map(draftZones);
    updated.set(monitorId, [...existing, zone]);
    draftZones = updated;
    selectedZoneId = id;
    selectedMonitorId = monitorId;
  }

  function handleSelectZone(zoneId: string | null) {
    selectedZoneId = zoneId;
    if (zoneId) {
      for (const [mid, zones] of draftZones) {
        if (zones.some(z => z.id === zoneId)) {
          selectedMonitorId = mid;
          break;
        }
      }
    }
  }

  function handleChangeZone(zoneId: string, patch: Partial<Zone>) {
    const updated = new Map(draftZones);
    for (const [mid, zones] of updated) {
      const idx = zones.findIndex(z => z.id === zoneId);
      if (idx !== -1) {
        const newZones = [...zones];
        newZones[idx] = { ...newZones[idx], ...patch };
        updated.set(mid, newZones);
        break;
      }
    }
    draftZones = updated;
  }

  function handleRenameZone(zoneId: string, name: string) {
    handleChangeZone(zoneId, { name });
  }

  function handleDeleteZone(zoneId: string) {
    const updated = new Map(draftZones);
    for (const [mid, zones] of updated) {
      const filtered = zones.filter(z => z.id !== zoneId);
      if (filtered.length !== zones.length) {
        updated.set(mid, filtered);
        break;
      }
    }
    draftZones = updated;
    if (selectedZoneId === zoneId) {
      selectedZoneId = null;
      selectedMonitorId = null;
    }
  }

  async function handleApply() {
    isApplying = true;
    errorMessage = "";
    try {
      for (const [monitorId, zones] of draftZones) {
        await applyLayout({ zones, monitor_id: monitorId });
      }
      const state = await getCurrentState();
      currentState.set(state);
      savedLayouts.set(state.saved_layouts);
      draftZones = new Map();
      for (const layout of state.active_layouts) {
        draftZones.set(layout.monitor_id, [...layout.zones]);
      }
      selectedZoneId = null;
      selectedMonitorId = null;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      errorMessage = `Apply failed: ${msg}`;
      notify(`Apply failed: ${msg}`, "error");
    } finally {
      isApplying = false;
    }
  }

  async function handleSave() {
    if (!saveName.trim()) return;
    isSaving = true;
    errorMessage = "";
    try {
      for (const [monitorId, zones] of draftZones) {
        await saveLayout(saveName, zones, monitorId);
      }
      saveName = "";
      const state = await getCurrentState();
      currentState.set(state);
      savedLayouts.set(state.saved_layouts);
      draftZones = new Map();
      for (const layout of state.active_layouts) {
        draftZones.set(layout.monitor_id, [...layout.zones]);
      }
      selectedZoneId = null;
      selectedMonitorId = null;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      errorMessage = `Save failed: ${msg}`;
      notify(`Save failed: ${msg}`, "error");
    } finally {
      isSaving = false;
    }
  }

  function selectedZone(): Zone | null {
    if (!selectedZoneId || !selectedMonitorId) return null;
    const zones = draftZones.get(selectedMonitorId);
    return zones?.find(z => z.id === selectedZoneId) ?? null;
  }
</script>

<div class="editor">
  <div class="toolbar">
    <div class="toolbar-left">
      <input
        class="save-input"
        type="text"
        placeholder="Layout name..."
        bind:value={saveName}
        disabled={isSaving}
        aria-label="Layout name"
      />
      <Button
        variant="primary"
        disabled={!saveName.trim() || isSaving}
        loading={isSaving}
        onclick={handleSave}
      >
        Save
      </Button>
      <Button
        variant="primary"
        disabled={isApplying || draftZones.size === 0}
        loading={isApplying}
        onclick={handleApply}
      >
        Apply Live
      </Button>
    </div>
    <div class="toolbar-right">
      {#if isApplying || isSaving}
        <Badge tone="warning" text="Working..." />
      {/if}
    </div>
  </div>

  {#if errorMessage}
    <div class="error-banner" role="alert">{errorMessage}</div>
  {/if}

  {#if monitors.length === 0}
    <EmptyState
      eyebrow="NO MONITORS"
      title="Connect a display"
      description="No monitors detected. Connect a display to begin creating zone layouts."
    />
  {:else}
    <div class="workspace-layout">
      <div class="monitors-view">
        {#each monitors as monitor (monitor.id)}
          <Panel title="{monitor.name} ({monitor.width}&times;{monitor.height})">
            <MonitorCanvas
              {monitor}
              draftZones={draftZones.get(monitor.id) ?? []}
              {selectedZoneId}
              onCreateZone={handleCreateZone}
              onSelectZone={handleSelectZone}
              onChangeZone={handleChangeZone}
              onDeleteZone={handleDeleteZone}
            />
          </Panel>
        {/each}
      </div>

      {#if selectedZone()}
        <div class="inspector-panel">
          <ZoneInspector
            zone={selectedZone()}
            onRename={handleRenameZone}
            onDelete={handleDeleteZone}
            onChange={handleChangeZone}
          />
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .save-input {
    height: var(--control-height);
    padding: 0 10px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    color: var(--text);
    font-family: var(--sans);
    font-size: 13px;
    width: 180px;
  }

  .save-input:focus {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .save-input:disabled {
    opacity: 0.45;
  }

  .error-banner {
    padding: 8px 12px;
    background: rgba(244, 67, 54, 0.1);
    border: 1px solid rgba(244, 67, 54, 0.3);
    border-radius: var(--radius-control);
    color: #e57373;
    font-size: 13px;
  }

  .workspace-layout {
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }

  .monitors-view {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .inspector-panel {
    width: 260px;
    flex-shrink: 0;
  }
</style>
