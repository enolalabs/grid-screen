<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentState } from "./lib/ipc";
  import { currentState, savedLayouts } from "./lib/stores";
  import { notify, toastNotifications } from "./lib/notifications";
  import AppShell from "./lib/components/AppShell.svelte";
  import LayoutEditor from "./routes/LayoutEditor.svelte";
  import LayoutManager from "./routes/LayoutManager.svelte";
  import Settings from "./routes/Settings.svelte";
  import type { AppView, InitializationState } from "./lib/types";
  import { listen } from "@tauri-apps/api/event";

  let activeView = $state<AppView>("workspace");
  let initialization = $state<InitializationState>({ status: "loading" });
  let toasts = $state<Array<{ id: string; message: string; type: "info" | "warning" | "error" }>>([]);

  async function loadState() {
    initialization = { status: "loading" };
    try {
      const state = await getCurrentState();
      currentState.set(state);
      savedLayouts.set(state.saved_layouts);
      settings.set(state.settings);
      initialization = { status: "loaded", state };

      if (!state.settings.first_run_completed) {
        activeView = "workspace";
      } else if (state.saved_layouts.length === 0) {
        activeView = "workspace";
      } else if (state.monitors.length === 0) {
        activeView = "status";
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      initialization = { status: "failed", message };
    }
  }

  onMount(async () => {
    const unsubToasts = toastNotifications.subscribe(n => toasts = n);

    const unlistenEvent = await listen<{ level: string; message: string }>(
      "user-notification",
      (event) => {
        const { level, message } = event.payload;
        notify(message, level as "info" | "warning" | "error");
      },
    );

    await loadState();

    return () => {
      unsubToasts();
      unlistenEvent();
    };
  });

  function handleNavigate(view: AppView) {
    activeView = view;
  }

  async   function handleRetry() {
    await loadState();
  }
</script>

<AppShell
  activeView={activeView}
  {initialization}
  isPaused={initialization.status === "loaded" ? initialization.state.is_paused : false}
  monitorCount={initialization.status === "loaded" ? initialization.state.monitors.length : 0}
  onNavigate={handleNavigate}
  onRetry={handleRetry}
>
  {#if initialization.status === "loaded"}
    {#if activeView === "workspace"}
      <LayoutEditor />
    {:else if activeView === "layouts"}
      <LayoutManager />
    {:else if activeView === "settings"}
      <Settings />
    {:else if activeView === "status"}
      <div class="status-page">
        <h2>System Status</h2>
        <p>Monitors: {initialization.state.monitors.length}</p>
        <p>Saved layouts: {initialization.state.saved_layouts.length}</p>
        <p>Paused: {initialization.state.is_paused ? "Yes" : "No"}</p>
      </div>
    {/if}
  {/if}
</AppShell>

<div class="toast-container" role="status" aria-live="polite">
  {#each toasts as n (n.id)}
    <div class="toast toast-{n.type}">{n.message}</div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 200;
  }

  .toast {
    padding: 10px 20px;
    border-radius: var(--radius-control);
    font-family: var(--sans);
    font-size: 13px;
    animation: slideIn 0.3s ease;
  }

  .toast-info {
    background: var(--surface-3);
    color: var(--text);
  }

  .toast-warning {
    background: rgba(255, 167, 38, 0.15);
    color: #ffb74d;
    border: 1px solid rgba(255, 167, 38, 0.3);
  }

  .toast-error {
    background: rgba(244, 67, 54, 0.15);
    color: #e57373;
    border: 1px solid rgba(244, 67, 54, 0.3);
  }

  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  @media (prefers-reduced-motion: reduce) {
    .toast { animation: none; }
  }

  .status-page {
    padding: 16px 0;
  }

  .status-page h2 {
    margin: 0 0 12px;
    font-size: 16px;
    font-weight: 600;
  }

  .status-page p {
    margin: 4px 0;
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
