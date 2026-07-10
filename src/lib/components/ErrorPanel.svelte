<script lang="ts">
  interface Props {
    title: string;
    message: string;
    retry?: () => void;
  }

  let { title, message, retry }: Props = $props();
</script>

<div class="error-panel" role="alert">
  <span class="error-icon" aria-hidden="true">!</span>
  <div class="error-content">
    <h4 class="error-title">{title}</h4>
    <p class="error-message">{message}</p>
  </div>
  {#if retry}
    <button class="btn-retry" onclick={retry}>Retry</button>
  {/if}
</div>

<style>
  .error-panel {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 16px;
    background: rgba(244, 67, 54, 0.08);
    border: 1px solid rgba(244, 67, 54, 0.25);
    border-radius: var(--radius-panel);
  }

  .error-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    border-radius: 50%;
    background: rgba(244, 67, 54, 0.2);
    color: #e57373;
    font-family: var(--sans);
    font-size: 13px;
    font-weight: 600;
    line-height: 1;
  }

  .error-content {
    flex: 1;
    min-width: 0;
  }

  .error-title {
    margin: 0 0 4px;
    font-size: 14px;
    font-weight: 600;
    color: #e57373;
  }

  .error-message {
    margin: 0;
    font-size: 13px;
    color: var(--text-muted);
    line-height: 1.4;
  }

  .btn-retry {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: var(--control-height);
    padding: 0 12px;
    background: transparent;
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    font-family: var(--sans);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease;
    white-space: nowrap;
  }

  @media (prefers-reduced-motion: reduce) {
    .btn-retry {
      transition: none;
    }
  }

  .btn-retry:hover {
    background: var(--surface-3);
    border-color: var(--surface-4);
  }

  .btn-retry:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }
</style>
