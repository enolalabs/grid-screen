<script lang="ts">
  import type { AppView } from "../types";

  interface Props {
    activeView: AppView;
    isPaused: boolean;
    onNavigate: (view: AppView) => void;
  }

  let { activeView, isPaused, onNavigate }: Props = $props();

  const navItems: { view: AppView; label: string; labelStatus?: string }[] = [
    { view: "workspace", label: "Workspace" },
    { view: "layouts", label: "Saved Layouts" },
    { view: "settings", label: "Settings" },
  ];

  function handleKeydown(e: KeyboardEvent, view: AppView) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onNavigate(view);
    }
  }
</script>

<aside class="sidebar" role="navigation" aria-label="Main navigation">
  <div class="sidebar-header">
    <span class="logo">Grid Screen</span>
  </div>

  <nav class="nav-list">
    {#each navItems as item (item.view)}
      {#if activeView === "status"}
        <button
          class="nav-item"
          tabindex="0"
          role="tab"
          onclick={() => onNavigate(item.view)}
          onkeydown={(e) => handleKeydown(e, item.view)}
        >
          {item.label}
        </button>
      {:else}
        <button
          class="nav-item"
          class:active={activeView === item.view}
          aria-selected={activeView === item.view}
          tabindex="0"
          role="tab"
          onclick={() => onNavigate(item.view)}
          onkeydown={(e) => handleKeydown(e, item.view)}
        >
          {item.label}
        </button>
      {/if}
    {/each}
  </nav>

  <div class="sidebar-footer">
    {#if activeView === "status"}
      <button
        class="nav-item back-btn"
        onclick={() => onNavigate("workspace")}
        onkeydown={(e) => handleKeydown(e, "workspace")}
        tabindex="0"
      >
        Back to Workspace
      </button>
    {:else}
      <button
        class="nav-item status-btn"
        class:active={activeView === "status"}
        aria-selected={activeView === "status"}
        onclick={() => onNavigate("status")}
        onkeydown={(e) => handleKeydown(e, "status")}
        tabindex="0"
        role="tab"
      >
        <span class="status-dot" class:active-dot={!isPaused} class:paused-dot={isPaused}></span>
        System Status
        {#if isPaused}
          <span class="paused-label">Paused</span>
        {/if}
      </button>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 280px;
    min-width: 280px;
    height: 100vh;
    background: var(--surface-1);
    border-right: 1px solid var(--border);
    user-select: none;
  }

  .sidebar-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
  }

  .logo {
    font-family: var(--sans);
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .nav-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    flex: 1;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    color: var(--text-muted);
    border: none;
    border-radius: var(--radius-control);
    font-family: var(--sans);
    font-size: 13px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  @media (prefers-reduced-motion: reduce) {
    .nav-item { transition: none; }
  }

  .nav-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .nav-item:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .nav-item.active {
    background: rgba(139, 92, 246, 0.12);
    color: var(--primary-bright);
  }

  .sidebar-footer {
    padding: 8px;
    border-top: 1px solid var(--border);
  }

  .status-btn {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .back-btn {
    color: var(--text-muted);
  }

  .back-btn:hover {
    color: var(--text);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .active-dot {
    background: #81c784;
  }

  .paused-dot {
    background: #ffb74d;
  }

  .paused-label {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #ffb74d;
    margin-left: auto;
  }
</style>
