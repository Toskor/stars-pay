<script lang="ts">
  import type { Page, Bot } from "./types";
  import { onDestroy, onMount } from "svelte";
  import {
    Button,
    Divider,
    Title,
    Text,
    Image,
    Input,
    EditIcon,
    QuestionMarkIcon,
    Section,
    SectionFooter,
  } from "telegram-ui";
  import { changeBotToken } from "./queries";
  import { botsStore } from "./store";

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
        navigateTo("edit", bot || undefined);
      });
    }
  });

  onDestroy(() => {
    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("main");
      });
    }
  });

  let value = $state("");
  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");

  let errorMessage = $state<string | null>(null);
  let isLoading = $state(false);

  const handleTokenChange = async () => {
    if (isLoading || !bot || !app) return;

    if (!value.trim()) {
      errorMessage = "Bot token cannot be empty";
      return;
    }

    isLoading = true;
    errorMessage = null;

    console.log("handleTokenChange", bot.id, value);

    try {
      const res = await changeBotToken(app.initData, bot.id, value);

      isLoading = false;

      if (res.success) {
        app.showPopup({
          title: "Success",
          message: "Bot token successfully updated!",
          buttons: [{ type: "ok" }],
        });

        navigateTo("edit", bot);
      } else {
        errorMessage = res.error || "Failed to update bot token";
      }
    } catch (err: unknown) {
      isLoading = false;
      errorMessage =
        err instanceof Error ? err.message : "An unexpected error occurred";
    }
  };
</script>

<div class="layout">
  <Image
    src={bot?.avatar ||
      "https://avatars.mds.yandex.net/i?id=0b56680182693c18b90b7e5047abbe27db7bd586-9198264-images-thumbs&n=13"}
    size={128}
  ></Image>

  <div class="layout-horizontal">
    <Title weight={1} level={3}>Change Bot Token</Title>
    <QuestionMarkIcon />
  </div>

  <Input
    placeholder="New bot token"
    header="Input"
    bind:value
    stretched={true}
  />

  {#if errorMessage}
    <div style="color: var(--tgui--destructive_text_color); margin-top: 8px;">
      <Text>{errorMessage}</Text>
    </div>
  {/if}

  <SectionFooter centered={false}>
    <Text>
      Changing the bot token will update the connection to your bot.
      <br />
      Make sure you have the correct token from
      <a
        href="https://t.me/botfather"
        style="color: var(--tgui--link_color); cursor: pointer;"
      >
        @botfather
      </a>
      <br />
      This action will not affect your bot's data or settings.
    </Text>
  </SectionFooter>

  <Button
    mode="filled"
    size="l"
    stretched={true}
    loading={isLoading}
    onclick={handleTokenChange}
  >
    {isLoading ? "Updating..." : "Update Token"}

    {#snippet before()}
      <EditIcon />
    {/snippet}
  </Button>
</div>

<style>
</style>
