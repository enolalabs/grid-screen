<script lang="ts">
  import { onMount } from "svelte";
  import { currentState, savedLayouts } from "../lib/stores";
  import { listLayouts, deleteLayout, setDefaultLayout, getCurrentState } from "../lib/ipc";
  import type { SavedLayout, FrontendState } from "../lib/types";
  import { isDefaultLayout, isActiveLayout } from "../lib/view-models";
  import { notify } from "../lib/notifications";
  import Panel from "../lib/components/Panel.svelte";
  import Badge from "../lib/components/Badge.svelte";
  import Button from "../lib/components/Button.svelte";
  import LayoutThumbnail from "../lib/components/LayoutThumbnail.svelte";
  import EmptyState from "../lib/components/EmptyState.svelte";

  let layouts = $state<SavedLayout[]>([]);
  let state = $state<FrontendState | null>(null);
  let pendingSetDefault = $state<string | null>(null);
  let pendingDelete = $state<string | null>(null);

  const unsubCurrentState = currentState.subscribe(value => {
    state = value;
  });

  onMount(async () => {
    layouts = await listLayouts();
  });

  function getActiveLayoutMonitorIds(): string[] {
    if (!state) return [];
    return state.active_layouts.map(al => al.monitor_id);
  }

  async function handleSetDefault(layout: SavedLayout) {
    if (pendingSetDefault) return;
    pendingSetDefault = layout.id;
    try {
      await setDefaultLayout(layout.id);
      const fresh = await getCurrentState();
      currentState.set(fresh);
      savedLayouts.set(fresh.saved_layouts);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      notify(`Failed to set default: ${message}`, "error");
    } finally {
      pendingSetDefault = null;
    }
  }

  async function handleDelete(layout: SavedLayout) {
    if (pendingDelete) return;
    if (!confirm(`Delete layout "${layout.name}"?`)) return;
    pendingDelete = layout.id;
    try {
      await deleteLayout(layout.id);
      layouts = await listLayouts();
      const fresh = await getCurrentState();
      currentState.set(fresh);
      savedLayouts.set(fresh.saved_layouts);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      notify(`Failed to delete: ${message}`, "error");
    } finally {
      pendingDelete = null;
    }
  }
</script>

<div class="layout-manager">
  <h2 class="page-title">Saved Layouts</h2>

  {#if layouts.length === 0}
    <EmptyState
      eyebrow="No Layouts"
      title="No layouts saved"
      description="Create a layout in the Workspace to see it here. Saved layouts let you quickly switch between zone arrangements."
      actionLabel="Go to Workspace"
      onAction={() => {
        const event = new CustomEvent("navigate", { detail: "workspace" });
        window.dispatchEvent(event);
      }}
    />
  {:else}
    <div class="layout-grid">
      {#each layouts as layout (layout.id)}
        <Panel title={layout.name} eyebrow="Layout">
          {#snippet children()}
            <LayoutThumbnail layoutId={layout.id} zones={layout.zones} label={layout.name} />
            <div class="layout-meta">
              <span class="meta-zones">{layout.zones.length} {layout.zones.length === 1 ? "zone" : "zones"}</span>
              <span class="meta-monitor">{layout.monitor_id}</span>
            </div>
            {#if state && isDefaultLayout(layout, state.settings)}
              <Badge tone="primary" text="Default" />
            {/if}
            {#if state && isActiveLayout(layout, state.active_layouts)}
              <Badge tone="success" text="Active" />
            {/if}
            <div class="layout-actions">
              <Button
                variant="ghost"
                disabled={pendingSetDefault === layout.id || (state !== null && isDefaultLayout(layout, state.settings))}
                loading={pendingSetDefault === layout.id}
                onclick={() => handleSetDefault(layout)}
              >
                Set Default
              </Button>
              <Button
                variant="danger"
                disabled={pendingDelete === layout.id}
                loading={pendingDelete === layout.id}
                onclick={() => handleDelete(layout)}
              >
                Delete
              </Button>
            </div>
          {/snippet}
        </Panel>
      {/each}
    </div>
  {/if}
</div>

<style>
  .layout-manager {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .page-title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .layout-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 12px;
  }

  .layout-meta {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    margin-bottom: 8px;
  }

  .meta-zones,
  .meta-monitor {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .layout-actions {
    display: flex;
    gap: 6px;
    margin-top: 12px;
  }

  :global(.layout-gap) {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
</style>
