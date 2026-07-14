<script lang="ts">
  interface Props {
    value: string;
  }

  let { value = $bindable("") }: Props = $props();

  function onInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value;
  }
</script>

<input
  type="text"
  class="search-box"
  placeholder="Search windows..."
  value={value}
  oninput={onInput}
/>

<style>
  .search-box {
    width: 100%; padding: 8px 12px;
    background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--radius-sm); color: var(--text); font-size: 13px;
    font-family: inherit; outline: none; transition: var(--transition);
  }
  .search-box:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .search-box::placeholder { color: var(--text-mute); }
</style>
