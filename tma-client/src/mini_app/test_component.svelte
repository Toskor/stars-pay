<script lang="ts">
  import { onMount } from "svelte";

  onMount(() => {
    console.log("test component mounted");
  });
  let t = $state(0);
</script>

<div>test component</div>
<button onclick={() => t++}>click me</button>
<p>{t}</p>
