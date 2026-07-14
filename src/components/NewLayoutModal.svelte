<script lang="ts">
  import type { Layout, LayoutType } from "$lib/shared-types";
  import { showToast } from "$lib/stores/toasts";

  interface Props {
    onClose: () => void;
    onSave: (layout: Layout) => void;
  }

  let { onClose, onSave }: Props = $props();

  let name = $state("");
  let layoutType = $state<LayoutType>("preset");
  let zoneCount = $state(2);
  let columnsPattern = $state("1fr 1fr");

  const zonePresets: Record<number, string[]> = {
    2: ["1fr 1fr", "3fr 1fr", "1fr 3fr", "2fr 1fr"],
    3: ["1fr 1fr 1fr", "2fr 1fr 1fr", "1fr 2fr 1fr", "1fr 1fr 2fr"],
  };

  function handleZoneChange(e: Event) {
    const val = Number((e.target as HTMLSelectElement).value);
    zoneCount = val;
    columnsPattern = zonePresets[val]?.[0] ?? "1fr 1fr";
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function save() {
    const trimmed = name.trim();
    if (trimmed.length < 1 || trimmed.length > 64) {
      showToast("Name must be 1-64 characters", "error");
      return;
    }
    const now = new Date().toISOString();
    const newLayout: Layout = {
      id: crypto.randomUUID(),
      name: trimmed,
      type: layoutType,
      zones: zoneCount,
      columns: columnsPattern,
      rows: null,
      span_first: columnsPattern === "2fr 1fr" ? true : null,
      ratio: null,
      gap_px: 10,
      margin_px: 16,
      created_at: now,
      updated_at: now,
    };
    onSave(newLayout);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="modal-backdrop" onclick={handleBackdropClick} role="dialog">
  <div class="modal">
    <div class="modal-header">
      <h3>New Layout</h3>
    </div>
    <div class="modal-body">
      <div class="field">
        <label class="field-label" for="layout-name">Name</label>
        <input
          id="layout-name"
          class="field-input"
          type="text"
          maxlength="64"
          placeholder="My Layout"
          bind:value={name}
        />
      </div>
      <div class="field">
        <label class="field-label" for="layout-type">Type</label>
        <select
          id="layout-type"
          class="field-select"
          bind:value={layoutType}
        >
          <option value="preset">Preset</option>
        </select>
      </div>
      <div class="field">
        <label class="field-label" for="zone-count">Zones</label>
        <select
          id="zone-count"
          class="field-select"
          value={zoneCount}
          onchange={handleZoneChange}
        >
          <option value="2">2</option>
          <option value="3">3</option>
        </select>
      </div>
      <div class="field">
        <label class="field-label" for="columns-pattern">Columns</label>
        <select
          id="columns-pattern"
          class="field-select"
          bind:value={columnsPattern}
        >
          {#each zonePresets[zoneCount] ?? [] as opt (opt)}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick={onClose}>Cancel</button>
      <button class="btn btn-primary" onclick={save}>Save Layout</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 300;
  }
  .modal {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    width: 400px;
    max-width: 90vw;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  }
  .modal-header {
    padding: 20px 24px 0;
  }
  .modal-header h3 {
    font-size: 16px;
    font-weight: 600;
  }
  .modal-body {
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-dim);
  }
  .field-input,
  .field-select {
    padding: 8px 12px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text);
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: var(--transition);
  }
  .field-input:focus,
  .field-select:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .modal-footer {
    padding: 0 24px 20px;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .btn {
    padding: 10px 28px;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: var(--transition);
    font-family: inherit;
  }
  .btn-primary {
    background: var(--accent);
    color: #fff;
    box-shadow: 0 0 24px var(--accent-glow);
  }
  .btn-primary:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 32px var(--accent-glow);
  }
  .btn-secondary {
    background: var(--surface-3);
    color: var(--text);
  }
  .btn-secondary:hover {
    background: var(--border);
  }
</style>
