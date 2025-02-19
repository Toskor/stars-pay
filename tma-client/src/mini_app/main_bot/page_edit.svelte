<script lang="ts">
  import { Button, Text, Title } from "telegram-ui";
  import type { Page } from "./types";
  let { navigateTo }: { navigateTo: (page: Page) => void } = $props();
</script>

<div class="page">
  <Title>Edit</Title>
  <Text>Edit bot</Text>
</div>
