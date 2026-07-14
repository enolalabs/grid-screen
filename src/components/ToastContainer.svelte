<script lang="ts">
  import { toasts, dismissToast } from "$lib/stores/toasts";
</script>

{#each $toasts as toast (toast.id)}
  <div class="toast show {toast.type}">
    <span>{toast.message}</span>
    {#if toast.durationMs === 0}
      <button class="dismiss" onclick={() => dismissToast(toast.id)}>x</button>
    {/if}
  </div>
{/each}

<style>
  .toast {
    position: fixed; bottom: 80px; left: 50%;
    transform: translateX(-50%); z-index: 200;
    background: var(--surface-2); color: var(--text);
    padding: 12px 24px; border-radius: var(--radius-sm);
    font-size: 14px; font-weight: 500;
    border: 1px solid var(--success);
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  }
  .toast.error { border-color: var(--danger); }
  .toast.warning { border-color: #d29922; }
  .dismiss {
    margin-left: 12px; background: none; border: none;
    color: var(--text-dim); cursor: pointer; font-size: 14px;
  }
</style>
