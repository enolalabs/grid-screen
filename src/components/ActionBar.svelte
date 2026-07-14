<script lang="ts">
  import { assignments, assignedCount, clearAssignments } from "$lib/stores/assignments";
  import { selectedLayout } from "$lib/stores/layout";
  import { selectedScreenId } from "$lib/stores/screen";
  import { showToast } from "$lib/stores/toasts";
  import { arrangeState } from "$lib/stores/arrangeState";
  import { commands } from "$lib/commands";

  let isArranging = $state(false);

  const zonesTotal = $derived($selectedLayout?.zones ?? 0);

  async function handleArrange() {
    if (isArranging || $assignedCount === 0) return;
    isArranging = true;
    arrangeState.set({ status: "validating" });
    try {
      const result = await commands.arrangeWindows({
        layout_id: $selectedLayout?.id ?? "",
        screen_id: $selectedScreenId,
        assignments: { ...$assignments },
      });
      arrangeState.set({ status: "completed", errors: result.results.filter((r) => r.status !== "moved").length });
      showToast(`Arranged ${$assignedCount} window${$assignedCount > 1 ? "s" : ""} on screen`);
    } catch (e) {
      arrangeState.set({ status: "failed", reason: String(e) });
      showToast(String(e), "error");
    } finally {
      isArranging = false;
    }
  }

  function handleClear() {
    clearAssignments();
    showToast("All assignments cleared");
  }
</script>

<div class="action-bar">
  <div class="action-info">
    {#if $assignedCount === 0}
      No windows assigned
    {:else}
      <strong>{$assignedCount}</strong> of {zonesTotal} zones assigned
    {/if}
  </div>
  <div>
    <button class="btn btn-secondary" onclick={handleClear}>Clear All</button>
    <button
      class="btn btn-primary"
      disabled={$assignedCount === 0 || isArranging}
      onclick={handleArrange}
    >
      {isArranging ? "Arranging..." : `Arrange ${$assignedCount > 0 ? $assignedCount : ""} Window${$assignedCount !== 1 ? "s" : ""}`}
    </button>
  </div>
</div>

<style>
  .action-bar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 16px 20px; border-top: 1px solid var(--border); background: var(--surface);
  }
  .action-info { font-size: 13px; color: var(--text-dim); }
  .action-info :global(strong) { color: var(--accent); }
  .btn {
    padding: 10px 28px; border: none; border-radius: var(--radius-sm);
    font-size: 14px; font-weight: 600; cursor: pointer;
    transition: var(--transition); font-family: inherit;
  }
  .btn-primary {
    background: var(--accent); color: #fff;
    box-shadow: 0 0 24px var(--accent-glow);
  }
  .btn-primary:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 32px var(--accent-glow);
  }
  .btn-primary:disabled {
    opacity: 0.4; cursor: not-allowed; transform: none; box-shadow: none;
  }
  .btn-secondary {
    background: var(--surface-3); color: var(--text); margin-right: 8px;
  }
  .btn-secondary:hover { background: var(--border); }
</style>
