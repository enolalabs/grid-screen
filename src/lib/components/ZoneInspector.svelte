<script lang="ts">
  import type { Zone } from "../types";
  import Button from "./Button.svelte";

  interface Props {
    zone: Zone | null;
    onRename: (zoneId: string, name: string) => void;
    onDelete: (zoneId: string) => void;
    onChange: (zoneId: string, patch: Partial<Zone>) => void;
  }

  let {
    zone,
    onRename,
    onDelete,
    onChange,
  }: Props = $props();

  let deleteConfirm = $state(false);
  let editName = $state("");

  $effect(() => {
    if (zone) {
      editName = zone.name;
      deleteConfirm = false;
    }
  });

  function handleNameSubmit() {
    if (!zone || !editName.trim()) return;
    onRename(zone.id, editName.trim().slice(0, 64));
  }

  function handleNameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleNameSubmit();
    if (e.key === "Escape" && zone) { editName = zone.name; }
  }
</script>

{#if zone}
  <div class="inspector" role="complementary" aria-label="Zone inspector">
    <div class="inspector-header">
      <h4 class="inspector-title">Zone Properties</h4>
      <span class="zone-id">{zone.id.slice(0, 8)}</span>
    </div>

    <label class="field">
      <span class="field-label">Name</span>
      <input
        class="field-input"
        type="text"
        bind:value={editName}
        onblur={handleNameSubmit}
        onkeydown={handleNameKeydown}
        maxlength={64}
        aria-label="Zone name"
      />
    </label>

    <div class="field-group">
      <label class="field field-half">
        <span class="field-label">X</span>
        <input class="field-input" type="text" value={zone.x.toFixed(4)} readonly tabindex="-1" />
      </label>
      <label class="field field-half">
        <span class="field-label">Y</span>
        <input class="field-input" type="text" value={zone.y.toFixed(4)} readonly tabindex="-1" />
      </label>
    </div>

    <div class="field-group">
      <label class="field field-half">
        <span class="field-label">Width</span>
        <input class="field-input" type="text" value={zone.width.toFixed(4)} readonly tabindex="-1" />
      </label>
      <label class="field field-half">
        <span class="field-label">Height</span>
        <input class="field-input" type="text" value={zone.height.toFixed(4)} readonly tabindex="-1" />
      </label>
    </div>

    <div class="field-group">
      <label class="field field-half">
        <span class="field-label">Gap (px)</span>
        <input
          class="field-input"
          type="number"
          min="0"
          max="64"
          value={zone.gap}
          oninput={(e: Event) => {
            if (!zone) return;
            const v = parseInt((e.target as HTMLInputElement).value, 10);
            if (!isNaN(v)) onChange(zone.id, { gap: v });
          }}
        />
      </label>
      <label class="field field-half">
        <span class="field-label">Margin (px)</span>
        <input
          class="field-input"
          type="number"
          min="0"
          max="64"
          value={zone.margin}
          oninput={(e: Event) => {
            if (!zone) return;
            const v = parseInt((e.target as HTMLInputElement).value, 10);
            if (!isNaN(v)) onChange(zone.id, { margin: v });
          }}
        />
      </label>
    </div>

    <div class="inspector-actions">
      {#if deleteConfirm}
        <div class="delete-confirm">
          <span class="confirm-text">Delete "{zone.name}"?</span>
          <Button variant="danger" onclick={() => onDelete(zone.id)}>Delete</Button>
          <Button variant="ghost" onclick={() => deleteConfirm = false}>Cancel</Button>
        </div>
      {:else}
        <Button variant="ghost" onclick={() => deleteConfirm = true}>Delete Zone</Button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .inspector {
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .inspector-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .inspector-title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }

  .zone-id {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--text-muted);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .field-input {
    height: 28px;
    padding: 0 8px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    color: var(--text);
    font-family: var(--mono);
    font-size: 12px;
  }

  .field-input:focus {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .field-input[readonly] {
    opacity: 0.6;
    cursor: default;
  }

  .field-group {
    display: flex;
    gap: 8px;
  }

  .field-half {
    flex: 1;
    min-width: 0;
  }

  .inspector-actions {
    padding-top: 4px;
    border-top: 1px solid var(--border);
  }

  .delete-confirm {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .confirm-text {
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
