<script lang="ts">
  import "telegram-ui/styles";
  import { defaultProps } from "../goal/types";
  import Goal from "../goal/goal.svelte";
  import { Input } from "telegram-ui";
</script>


<!-- <Input header="Goal title" placeholder="Enter goal title" /> -->

<Goal goalProps={defaultProps} />
