<script lang="ts">
  import type { Page, Bot } from "./types";
  import { onMount } from "svelte";
  import { Button, Title, Text, Section } from "telegram-ui";
  import "telegram-ui/styles";
  import Preview from "../stream_bot/app.svelte";
  import type { PreviewData } from "../stream_bot/types";

  let {
    navigateTo,
    bot,
  }: {
    navigateTo: (page: Page, bot?: Bot) => void;
    bot: Bot | undefined;
  } = $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  onMount(() => {
    if (!bot) {
      navigateTo("main");
    }

    console.log("bot prev data", bot?.preview_data);

    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("manage_donation_buttons", bot || undefined);
      });
    }
  });
</script>

<Preview preview_data={bot!.preview_data} />

<style>
</style>
