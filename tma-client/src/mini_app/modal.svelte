<script>
  let { showModal = $bindable(), header, children } = $props();

  let dialog = $state();

  $effect(() => {
    if (showModal) dialog.showModal();
  });
</script>

<dialog bind:this={dialog} onclose={() => (showModal = false)} class="modal">
  <div
    class="modal-box"
    tabindex="0"
    onclick={(e) => {
      if (e.target === dialog) dialog.close();
    }}
    role="dialog"
  >
    <form method="dialog">
      <button class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
        >✕</button
      >
    </form>
    {@render header?.()}
    
    {@render children?.()}
    

    <!-- <button class="btn btn-primary btn-outline absolute right-2 bottom-2" onclick={() => dialog.close()}>Close</button> -->
  </div>
</dialog>

<style>
  /* dialog {
    max-width: 32em;
    border-radius: 0.2em;
    border: none;
    padding: 0;
  } */
  /* dialog::backdrop {
    background: rgba(0, 0, 0, 0.3);
  } */
  /* dialog > div {
    padding: 1em;
  } */
  /* dialog[open] {
    animation: zoom 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  } */
  /* @keyframes zoom {
    from {
      transform: scale(0.95);
    }
    to {
      transform: scale(1);
    }
  } */
  /* dialog[open]::backdrop {
    animation: fade 0.2s ease-out;
  } */
  /* @keyframes fade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  } */
  /* button {
    display: block;
  } */
</style>
