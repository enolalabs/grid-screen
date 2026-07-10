<script lang="ts">
  import type { AppView, InitializationState } from "../types";
  import Sidebar from "./Sidebar.svelte";
  import TopBar from "./TopBar.svelte";
  import ErrorPanel from "./ErrorPanel.svelte";
  import EmptyState from "./EmptyState.svelte";
  import { getFirstRunState } from "../view-models";

  interface Props {
    activeView: AppView;
    initialization: InitializationState;
    isPaused: boolean;
    monitorCount: number;
    onNavigate: (view: AppView) => void;
    onRetry: () => void;
    children?: import("svelte").Snippet;
  }

  let {
    activeView,
    initialization,
    isPaused,
    monitorCount,
    onNavigate,
    onRetry,
    children,
  }: Props = $props();

  function viewTitle(view: AppView): string {
    switch (view) {
      case "workspace": return "Workspace";
      case "layouts": return "Saved Layouts";
      case "settings": return "Settings";
      case "status": return "System Status";
    }
  }
</script>

<div class="app-root">
  <Sidebar {activeView} {isPaused} {onNavigate} />

  <div class="app-main">
    <TopBar title={viewTitle(activeView)} {isPaused} />

    <main class="app-content">
      {#if initialization.status === "loading"}
        <div class="loading-state">
          <EmptyState
            eyebrow="INITIALIZING"
            title="Loading"
            description="Connecting to the Grid Screen service..."
          />
        </div>
      {:else if initialization.status === "failed"}
        <div class="error-state">
          <ErrorPanel
            title="Connection Failed"
            message={initialization.message}
            retry={onRetry}
          />
        </div>
      {:else if initialization.status === "loaded"}
        {@const runState = getFirstRunState(initialization.state)}
        {#if runState === "onboarding"}
          <div class="onboarding-state">
            <EmptyState
              eyebrow="FIRST RUN"
              title="Welcome to Grid Screen"
              description="Create your first zone layout and start organizing your workspace."
              actionLabel={monitorCount > 0 ? "Open Workspace" : "Connect a display"}
              completionLabel="Skip setup"
              onboarding={true}
              onAction={() => onNavigate("workspace")}
            />
          </div>
        {:else if runState === "empty"}
          <div class="empty-state-wrapper">
            <EmptyState
              eyebrow="NO LAYOUTS"
              title="No layouts saved"
              description="Create a layout in the Workspace and save it to get started."
              actionLabel="Open Workspace"
              onAction={() => onNavigate("workspace")}
            />
          </div>
        {:else if runState === "recovery"}
          <div class="recovery-state">
            <EmptyState
              eyebrow="NO DISPLAYS"
              title="No monitors detected"
              description="Connect a monitor or display to continue. Your layouts are saved."
              actionLabel="Check System Status"
              onAction={() => onNavigate("status")}
            />
          </div>
        {:else}
          {#if typeof children === "function"}
            {@render children()}
          {:else}
            {children}
          {/if}
        {/if}
      {/if}
    </main>
  </div>
</div>

<style>
  .app-root {
    display: flex;
    height: 100vh;
    background: var(--canvas);
    color: var(--text);
    font-family: var(--sans);
    overflow: hidden;
  }

  .app-main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .app-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }

  .loading-state,
  .error-state,
  .onboarding-state,
  .empty-state-wrapper,
  .recovery-state {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100%;
  }

  .error-state {
    align-items: flex-start;
    padding-top: 24px;
  }
</style>
