<script lang="ts">
  import { arrangeState, type ArrangeStatus } from "$lib/stores/arrangeState";
  import { showToast } from "$lib/stores/toasts";

  let lastCompletedStatus: { errors: number } | null = $state(null);
  let dismissTimeout: ReturnType<typeof setTimeout> | null = $state(null);

  $effect(() => {
    const state: ArrangeStatus = $arrangeState;
    if (state.status === "completed") {
      lastCompletedStatus = { errors: state.errors };
      if (dismissTimeout) clearTimeout(dismissTimeout);
      dismissTimeout = setTimeout(() => {
        arrangeState.set({ status: "idle" });
        lastCompletedStatus = null;
        const msg =
          state.errors === 0
            ? "All windows arranged successfully"
            : `Arranged with ${state.errors} error${state.errors !== 1 ? "s" : ""}`;
        showToast(msg, state.errors === 0 ? "success" : "warning");
      }, 2000);
    } else {
      lastCompletedStatus = null;
    }

    return () => {
      if (dismissTimeout) clearTimeout(dismissTimeout);
    };
  });

  function dismiss() {
    arrangeState.set({ status: "idle" });
    lastCompletedStatus = null;
    if (dismissTimeout) clearTimeout(dismissTimeout);
  }
</script>

{#if $arrangeState.status === "idle"}
  <!-- hidden -->
{:else if $arrangeState.status === "validating"}
  <div class="overlay">
    <div class="overlay-card">
      <div class="spinner"></div>
      <p>Validating...</p>
    </div>
  </div>
{:else if $arrangeState.status === "arranging"}
  <div class="overlay">
    <div class="overlay-card">
      <div class="spinner"></div>
      <p>Arranging window {$arrangeState.current} of {$arrangeState.total}...</p>
    </div>
  </div>
{:else if $arrangeState.status === "completed"}
  {#if lastCompletedStatus}
    <div class="overlay">
      <div class="overlay-card">
        {#if lastCompletedStatus.errors === 0}
          <svg class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 6L9 17l-5-5" />
          </svg>
          <p>All windows arranged successfully</p>
        {:else}
          <p>{lastCompletedStatus.errors} window{lastCompletedStatus.errors !== 1 ? "s" : ""} failed to move</p>
        {/if}
      </div>
    </div>
  {/if}
{:else if $arrangeState.status === "failed"}
  <div class="overlay">
    <div class="overlay-card">
      <svg class="error-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="15" y1="9" x2="9" y2="15" />
        <line x1="9" y1="9" x2="15" y2="15" />
      </svg>
      <p>{$arrangeState.reason}</p>
      <button class="dismiss-btn" onclick={dismiss}>Dismiss</button>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 400;
    pointer-events: all;
  }
  .overlay-card {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 32px 48px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  }
  .overlay-card p {
    font-size: 15px;
    font-weight: 500;
    color: var(--text);
  }
  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--surface-3);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .check-icon {
    width: 40px;
    height: 40px;
    color: var(--success);
  }
  .error-icon {
    width: 40px;
    height: 40px;
    color: var(--danger);
  }
  .dismiss-btn {
    padding: 8px 20px;
    background: var(--surface-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
    font-family: inherit;
  }
  .dismiss-btn:hover {
    background: var(--border);
  }
</style>
