<script lang="ts">
  import type { Page, Bot } from "./types";
  import { onMount } from "svelte";
  import { Button, Title, Text, Section } from "telegram-ui";
  import "telegram-ui/styles";
  import Preview from "../stream_bot/app.svelte";

  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | null } =
    $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  onMount(() => {
    if (!bot) {
      navigateTo("main");
    }

    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("manage_donation_buttons", bot || undefined);
      });
    }
  });
</script>

<Preview />

<style>
</style>
