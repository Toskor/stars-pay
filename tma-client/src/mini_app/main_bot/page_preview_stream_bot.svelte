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
    preview_data,
  }: {
    navigateTo: (page: Page, bot?: Bot) => void;
    bot: Bot | null;
    preview_data: PreviewData;
  } = $props();

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

<Preview preview_data={bot?.preview_data} />

<style>
</style>
