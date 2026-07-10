<script lang="ts">
  interface Props {
    variant?: "primary" | "ghost" | "danger";
    type?: "button" | "submit";
    disabled?: boolean;
    loading?: boolean;
    ariaLabel?: string;
    onclick?: (e: MouseEvent) => void;
    children?: import("svelte").Snippet;
  }

  let {
    variant = "primary",
    type = "button",
    disabled = false,
    loading = false,
    ariaLabel,
    onclick,
    children,
  }: Props = $props();
</script>

<button
  class="btn btn-{variant}"
  {type}
  disabled={disabled || loading}
  aria-label={ariaLabel}
  aria-busy={loading}
  {onclick}
>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
  {/if}
  <span class="btn-label" class:visually-hidden={loading}>
    {#if typeof children === "function"}
      {@render children()}
    {:else}
      {children}
    {/if}
  </span>
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: var(--control-height);
    padding: 0 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    font-family: var(--sans);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease, opacity 0.15s ease;
    white-space: nowrap;
    user-select: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .btn {
      transition: none;
    }
  }

  .btn:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--primary);
    color: #fff;
    border-color: var(--primary);
  }

  .btn-primary:hover:not(:disabled) {
    background: #7c4fe6;
    border-color: #7c4fe6;
  }

  .btn-ghost {
    background: transparent;
    color: var(--text);
    border-color: var(--border);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--surface-3);
    border-color: var(--surface-4);
  }

  .btn-danger {
    background: #d32f2f;
    color: #fff;
    border-color: #d32f2f;
  }

  .btn-danger:hover:not(:disabled) {
    background: #b71c1c;
    border-color: #b71c1c;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
