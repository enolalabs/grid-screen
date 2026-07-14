<script lang="ts">
  interface Props {
    title: string;
    children?: import("svelte").Snippet;
  }

  let { title, children }: Props = $props();
</script>

<div class="settings-group">
  <div class="settings-group-title">{title}</div>
  <div>
    {@render children?.()}
  </div>
</div>

<style>
  .settings-group {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    margin-bottom: 16px;
    overflow: hidden;
  }
  .settings-group-title {
    padding: 14px 20px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-mute);
    border-bottom: 1px solid var(--border);
  }
</style>
