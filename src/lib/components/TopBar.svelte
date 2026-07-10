<script lang="ts">
  interface Props {
    title: string;
    isPaused: boolean;
  }

  let { title, isPaused }: Props = $props();
</script>

<header class="topbar">
  <h1 class="topbar-title">{title}</h1>
  {#if isPaused}
    <span class="paused-badge" role="status">Paused</span>
  {/if}
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 16px;
    background: var(--surface-1);
    border-bottom: 1px solid var(--border);
  }

  .topbar-title {
    margin: 0;
    font-family: var(--sans);
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
  }

  .paused-badge {
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 0 8px;
    border-radius: var(--radius-control);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    background: rgba(255, 167, 38, 0.15);
    color: #ffb74d;
    border: 1px solid rgba(255, 167, 38, 0.3);
  }
</style>
