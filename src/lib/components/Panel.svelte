<script lang="ts">
  interface Props {
    title?: string;
    eyebrow?: string;
    interactive?: boolean;
    header?: import("svelte").Snippet;
    children?: import("svelte").Snippet;
  }

  let {
    title,
    eyebrow,
    interactive = false,
    header,
    children,
  }: Props = $props();
</script>

<div class="panel" class:interactive>
  {#if header}
    <div class="panel-header">{@render header()}</div>
  {:else if title || eyebrow}
    <div class="panel-header">
      {#if eyebrow}
        <span class="panel-eyebrow">{eyebrow}</span>
      {/if}
      {#if title}
        <h3 class="panel-title">{title}</h3>
      {/if}
    </div>
  {/if}
  <div class="panel-body">
    {#if typeof children === "function"}
      {@render children()}
    {:else}
      {children}
    {/if}
  </div>
</div>

<style>
  .panel {
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    overflow: hidden;
  }

  .panel.interactive {
    cursor: pointer;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  .panel.interactive:hover {
    border-color: var(--primary);
    box-shadow: 0 0 0 1px rgba(139, 92, 246, 0.2);
  }

  @media (prefers-reduced-motion: reduce) {
    .panel.interactive {
      transition: none;
    }
  }

  .panel-header {
    padding: 12px 16px 0;
  }

  .panel-eyebrow {
    display: block;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .panel-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .panel-body {
    padding: 12px 16px;
  }
</style>
