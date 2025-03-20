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
    AddCircleIcon,
  } from "telegram-ui";
  import { addAdmin } from "./queries";
  import { botsStore } from "./store";

  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | undefined } =
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

  let value = $state("");
  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");

  let errorMessage = $state<string | null>(null);
  let isLoading = $state(false);

  const handleAddAdmin = async () => {
    if (isLoading || !bot || !app) return;

    const inputValue = String(value).trim();

    if (!inputValue) {
      errorMessage = "Admin ID cannot be empty";
      return;
    }

    const adminId = parseInt(inputValue);
    if (isNaN(adminId)) {
      errorMessage = "Admin ID must be a valid number";
      return;
    }

    isLoading = true;
    errorMessage = null;

    try {
      const res = await addAdmin(app.initData, bot.id, adminId);

      isLoading = false;

      if (res.success && res.data) {
        // Update the store by adding the new admin to the current bot
        botsStore.update((store) => {
          if (store.data && store.data.bots) {
            const updatedBots = store.data.bots.map((storeBot) => {
              if (storeBot.id === bot.id) {
                const newAdmin = {
                  id: res.data.admin_info.id,
                  username: res.data.admin_info.username,
                  name: res.data.admin_info.name,
                  avatarUrl: res.data.admin_info.avatarUrl,
                };

                return {
                  ...storeBot,
                  admins: [...storeBot.admins, newAdmin],
                };
              }
              return storeBot;
            });

            return {
              ...store,
              data: {
                ...store.data,
                bots: updatedBots,
              },
            };
          }
          return store;
        });

        app.showPopup({
          title: "Success",
          message: "Admin successfully added to the bot!",
          buttons: [{ type: "ok" }],
        });

        navigateTo("edit", bot);
      } else {
        errorMessage = res.success
          ? "Failed to add admin"
          : JSON.parse(res.error).error;
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
    <Title weight={1} level={3}>Add Admin to Bot</Title>
  </div>

  <Input
    placeholder="Admin Telegram ID"
    header="Input"
    bind:value
    stretched={true}
    type="number"
  />

  {#if errorMessage}
    <div style="color: var(--tgui--destructive_text_color); margin-top: 8px;">
      <Text>{errorMessage}</Text>
    </div>
  {/if}

  <SectionFooter centered={false}>
    <Text>
      Adding an admin will give them permission to manage this bot.
      <br />
      You need to provide the Telegram user ID of the person you want to add.
      <br />
      The user must have interacted with the bot at least once.
    </Text>
  </SectionFooter>

  <Button
    mode="filled"
    size="l"
    stretched={true}
    loading={isLoading}
    onclick={handleAddAdmin}
  >
    {isLoading ? "Adding..." : "Add Admin"}

    {#snippet before()}
      <AddCircleIcon isFill={true} />
    {/snippet}
  </Button>
</div>

<style>
  .layout {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .layout-horizontal {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
  }
</style>
