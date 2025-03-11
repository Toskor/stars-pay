<script lang="ts">
  import type { Page, Bot, User } from "./types";
  import { onMount } from "svelte";
  import {
    Button,
    Divider,
    Title,
    Text,
    Image,
    Input,
    AddCircleIcon,
    QuestionMarkIcon,
    Section,
    SectionFooter,
  } from "telegram-ui";
  import { addBot } from "./queries";
  import { botsStore } from "./store";
  import { get } from "svelte/store";
  let { navigateTo }: { navigateTo: (page: Page, bot?: Bot) => void } =
    $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  onMount(() => {
    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("main");
      });
    }
  });

  let value = $state("");
  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");

  // Add states for error and loading
  let errorMessage = $state<string | null>(null);
  let isLoading = $state(false);
</script>

<div class="layout">
  <Image
    src="https://avatars.mds.yandex.net/i?id=0b56680182693c18b90b7e5047abbe27db7bd586-9198264-images-thumbs&n=13"
    size={128}
  ></Image>

  <div class="layout-horizontal">
    <Title weight={1} level={3}>Register new bot</Title>
    <QuestionMarkIcon />
  </div>

  <Input
    placeholder="Your bot token"
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
    Create bot with
    <a
      href="https://t.me/botfather"
      style="color: var(--tgui--link_color); cursor: pointer;"
    >
      @botfather</a
    >
    and paste here bot token
    <br />
    You will be owner of this bot by default
  </SectionFooter>

  <Button
    mode="filled"
    size="l"
    stretched={true}
    loading={isLoading}
    onclick={() => {
      if (isLoading) return;

      if (!value.trim()) {
        errorMessage = "Bot token cannot be empty";
        return;
      }

      isLoading = true;
      errorMessage = null;

      if (app) {
        addBot(app.initData, value)
          .then((res) => {
            isLoading = false;

          if (res.success && res.data && res.data.bot_data) {
            const newBot: Bot = {
              id: res.data.bot_data.id,
              numeric_id: res.data.bot_data.numeric_id,
              name: res.data.bot_data.name,
              avatar: res.data.bot_data.avatar || "",
              userRole: res.data.bot_data.userRole,
              owner: res.data.bot_data.owner,
              admins: res.data.bot_data.admins || [],
            };

            botsStore.update((store) => {
              if (!store.data) {
                return {
                  ...store,
                  isLoaded: true,
                  isLoading: false,
                  error: null,
                  data: { bots: [newBot] },
                  loadTime: performance.now(),
                };
              }

              return {
                ...store,
                data: {
                  ...store.data,
                  bots: [...store.data.bots, newBot],
                },
              };
            });

            app.showPopup({
              title: "Success",
              message: "Bot successfully registered!",
              buttons: [{ type: "ok" }],
            });

            navigateTo("main");
          } else {
            errorMessage =
              res.success === false
                ? JSON.parse(res.error).error
                : "Failed to register bot";
          }
        })
        .catch((err) => {
          isLoading = false;
            errorMessage = err.message || "An unexpected error occurred";
          });
      }
    }}
  >
    {isLoading ? "Adding..." : "Add Bot"}

    {#snippet before()}
      <AddCircleIcon isFill={true} />
    {/snippet}
  </Button>
</div>
