<script lang="ts">
  import type { FrontendState, InitializationState, Notification, AppView } from "../lib/types";
  import Panel from "../lib/components/Panel.svelte";
  import StatusRow from "../lib/components/StatusRow.svelte";
  import ErrorPanel from "../lib/components/ErrorPanel.svelte";
  import EmptyState from "../lib/components/EmptyState.svelte";
  import Button from "../lib/components/Button.svelte";

  interface Props {
    state: FrontendState | null;
    initialization: InitializationState;
    history: Notification[];
    onRetry: () => void;
    onClearHistory: () => void;
    onNavigate: (view: AppView) => void;
  }

  let { state, initialization, history, onRetry, onClearHistory, onNavigate }: Props = $props();
</script>

<div class="system-status">
  <div class="status-header">
    <h2 class="status-title">System Status</h2>
    <Button variant="ghost" onclick={() => onNavigate("workspace")}>Back to Workspace</Button>
  </div>

  {#if initialization.status === "loading"}
    <div class="loading-state">
      <EmptyState eyebrow="LOADING" title="Loading" description="Connecting to Grid Screen..." />
    </div>
  {:else if initialization.status === "failed"}
    <ErrorPanel title="Connection Failed" message={initialization.message} retry={onRetry} />
  {:else if initialization.status === "loaded" && state}
    <div class="status-sections">
      <Panel title="Runtime">
        <StatusRow label="Monitors" value={String(state.monitors.length)} tone="primary" />
        <StatusRow label="Saved layouts" value={String(state.saved_layouts.length)} tone="primary" />
        <StatusRow
          label="Paused"
          value={state.is_paused ? "Yes" : "No"}
          tone={state.is_paused ? "warning" : "muted"}
        />
      </Panel>

      <Panel title="Recent Notifications">
        {#if history.length === 0}
          <p class="empty-hint">No recent notifications</p>
        {:else}
          <ul class="notification-list">
            {#each history as n}
              <li class="notification-item tone-{n.type}">
                <span class="notification-message">{n.message}</span>
                <span class="notification-type">{n.type}</span>
              </li>
            {/each}
          </ul>
        {/if}
        {#if history.length > 0}
          <div class="history-actions">
            <Button variant="ghost" onclick={onClearHistory}>Clear</Button>
          </div>
        {/if}
      </Panel>
    </div>
  {/if}
</div>

<style>
  .system-status {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .status-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .status-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .status-sections {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 200px;
  }

  .empty-hint {
    margin: 0;
    font-size: 13px;
    color: var(--text-muted);
    padding: 8px 0;
  }

  .notification-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .notification-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 0;
    font-size: 13px;
  }

  .notification-message {
    color: var(--text);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .notification-type {
    font-family: var(--mono);
    font-size: 11px;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .tone-info .notification-type { color: var(--primary-bright); }
  .tone-warning .notification-type { color: #ffb74d; }
  .tone-error .notification-type { color: #e57373; }

  .history-actions {
    padding-top: 8px;
  }
</style>
