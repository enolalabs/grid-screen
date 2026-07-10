<script lang="ts">
  import Badge from "./Badge.svelte";

  interface Props {
    title: string;
    isPaused: boolean;
  }

  let { title, isPaused }: Props = $props();
</script>

<header class="topbar">
  <h1 class="topbar-title">{title}</h1>
  {#if isPaused}
    <Badge tone="warning" text="Paused" />
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

</style>
