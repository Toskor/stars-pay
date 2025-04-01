<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";

  import {
    Button,
    Text,
    Title,
    Accordion,
    AccordionSummary,
    AccordionContent,
    Avatar,
    Cell,
    List,
    Section,
    SectionHeader,
    Divider,
    ForwardIcon,
    EditIcon,
    DeleteIcon,
    RetryIcon,
  } from "telegram-ui";
  import type { Page, Bot } from "./types";
  import { removeBotAdmin, removeBot, refreshLayerToken } from "./queries";
  import { botsStore, refreshBotsData } from "./store";
  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | undefined } =
    $props();

  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");
  let isRemoving = $state<number | null>(null);
  let isDeletingBot = $state<boolean>(false);
  let isRefreshingLayerToken = $state<boolean>(false);
  let currentBot = $state<Bot | undefined>(bot || undefined);

  //@ts-ignore
  let app = window.Telegram.WebApp;

  onMount(() => {
    if (!currentBot) {
      navigateTo("main");
      return;
    }

    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("main");
      });
    }

    const unsubscribe = botsStore.subscribe((store) => {
      if (store.data && store.data.bots && currentBot) {
        const botId = currentBot.id;
        const updatedBot = store.data.bots.find((b) => b.id === botId);
        if (updatedBot) {
          currentBot = updatedBot;
        }
      }
    });

    return unsubscribe;
  });

  async function removeAdmin(adminId: number, adminName: string) {
    if (!app || !currentBot) return;

    isRemoving = adminId;
    try {
      const botId = currentBot.id; // Сохраняем ID в локальную переменную
      const response = await removeBotAdmin(app.initData, botId, adminId);

      if (response.success) {
        // Update the store by removing the admin from the current bot
        botsStore.update((store) => {
          if (store.data && store.data.bots) {
            const updatedBots = store.data.bots.map((storeBot) => {
              if (currentBot && storeBot.id === currentBot.id) {
                return {
                  ...storeBot,
                  admins: storeBot.admins.filter((a) => a.id !== adminId),
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

        // Show success message
        app.showPopup({
          title: "Success",
          message: `Admin ${adminName} removed successfully`,
          buttons: [{ type: "ok" }],
        });
      } else {
        // Show error message
        app.showPopup({
          title: "Error",
          message: `Failed to remove admin: ${response.error}`,
          buttons: [{ type: "ok" }],
        });
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Unknown error";
      app.showPopup({
        title: "Error",
        message: `An error occurred: ${errorMessage}`,
        buttons: [{ type: "ok" }],
      });
    } finally {
      isRemoving = null;
    }
  }

  async function handleRemoveBot() {
    if (!app || !currentBot) return;

    const botName = currentBot.name;
    const botId = currentBot.id;

    // Show confirmation popup
    app.showConfirm(
      `Are you sure you want to remove bot "${botName}"? This action cannot be undone.`,
      async (confirmed: boolean) => {
        if (confirmed) {
          try {
            isDeletingBot = true;
            const response = await removeBot(app.initData, botId);

            if (response.success) {
              // Update the store by removing the bot
              botsStore.update((store) => {
                if (store.data && store.data.bots) {
                  return {
                    ...store,
                    data: {
                      ...store.data,
                      bots: store.data.bots.filter((b) => b.id !== botId),
                    },
                  };
                }
                return store;
              });

              // Show success message
              app.showPopup({
                title: "Success",
                message: `Bot ${botName} removed successfully`,
                buttons: [{ type: "ok" }],
              });

              // Navigate back to main page
              navigateTo("main");
            } else {
              // Show error message
              app.showPopup({
                title: "Error",
                message: `Failed to remove bot: ${response.error}`,
                buttons: [{ type: "ok" }],
              });
            }
          } catch (error) {
            const errorMessage =
              error instanceof Error ? error.message : "Unknown error";
            app.showPopup({
              title: "Error",
              message: `An error occurred: ${errorMessage}`,
              buttons: [{ type: "ok" }],
            });
          } finally {
            isDeletingBot = false;
          }
        }
      }
    );
  }

  function handleRefreshLayerToken() {
    app?.showPopup({
      title: "Refresh Layer Token",
      message:
        "Are you sure you want to refresh the layer token? This action disables all layers and requires an update to the Layer URL.",
      buttons: [
        {
          type: "cancel",
        },
        {
          type: "ok",
          onclick: () => {
            queryRefreshLayerToken();
          },
        },
      ],
    });
  }

  async function queryRefreshLayerToken() {
    if (!app || !currentBot) return;

    const botId = currentBot.id;
    isRefreshingLayerToken = true;
    const response = await refreshLayerToken(app.initData, botId);
    if (response.success) {
      app.showPopup({
        title: "Success",
        message: "Layer token refreshed successfully",
        buttons: [{ type: "ok" }],
      });
    } else {
      app.showPopup({
        title: "Error",
        message: "Failed to refresh layer token",
        buttons: [{ type: "ok" }],
      });
    }
    isRefreshingLayerToken = false;
  }
</script>

<List>
  <div class="header--center-container">
    <Avatar
      size={128}
      src={currentBot?.avatar || ""}
      acronym={currentBot?.name?.[0] || ""}
    />
    <Title level={3} weight={1}>{currentBot?.name || ""}</Title>
  </div>

  <Section>
    <Cell onclick={() => navigateTo("manage_donation_buttons", currentBot)}>
      Manage donation buttons
      {#snippet after()}
        <ForwardIcon />
      {/snippet}
    </Cell>
  </Section>

  <Section header="Owner">
    <Cell>
      {#snippet before()}
        <Avatar
          size={48}
          src={currentBot?.owner?.avatarUrl || ""}
          acronym={currentBot?.owner?.name?.[0] || ""}
        />
      {/snippet}

      {#snippet children()}
        {currentBot?.owner?.name || ""}
      {/snippet}

      {#snippet subtitle()}
        {currentBot?.owner?.username || ""}
      {/snippet}
    </Cell>
  </Section>

  <Section>
    {#snippet header()}
      <div class="header-row">
        <SectionHeader>Bot admins</SectionHeader>
        <Button
          mode="plain"
          size="s"
          onclick={() => navigateTo("add_admin", currentBot)}>Add</Button
        >
      </div>
    {/snippet}

    {#if currentBot?.admins && currentBot.admins.length > 0}
      {#each currentBot.admins as admin, index}
        <Cell>
          {#snippet before()}
            <Avatar
              size={48}
              src={admin.avatarUrl || ""}
              acronym={admin.name?.[0] || ""}
            />
          {/snippet}

          {#snippet after()}
            {#if currentBot?.userRole === "owner"}
              <Button
                mode="destructive"
                size="s"
                loading={isRemoving === admin.id}
                onclick={() => removeAdmin(admin.id, admin.name)}
              >
                Remove
              </Button>
            {/if}
          {/snippet}

          {#snippet children()}
            {admin.name || ""}
          {/snippet}

          {#snippet subtitle()}
            {admin.username || ""}
          {/snippet}
        </Cell>

        {#if index < currentBot.admins.length - 1}
          <Divider />
        {/if}
      {/each}
    {:else}
      <Cell>
        <div class="no-admins">No admins added yet</div>
      </Cell>
    {/if}
  </Section>

  <!-- Danger Zone -->
  {#if currentBot?.userRole === "owner"}
    <Section>
      <Accordion>
        <AccordionSummary>
          <Text weight={3}>Danger Zone</Text>
        </AccordionSummary>

        <AccordionContent style="color: var(--tgui--destructive_text_color);">
          <Cell
            onclick={() => {
              console.log("change owner");
            }}
          >
            {#snippet before()}
              <ForwardIcon />
            {/snippet}
            Change Owner
          </Cell>

          <Divider />

          <Cell
            onclick={() => {
              navigateTo("change_token", currentBot);
            }}
          >
            {#snippet before()}
              <EditIcon />
            {/snippet}
            Change Bot Token
          </Cell>

          <Divider />

          <Cell
            onclick={() => {
              handleRefreshLayerToken();
            }}
          >
            {#snippet before()}
              <RetryIcon />
            {/snippet}
            {#if isRefreshingLayerToken}
              Refreshing...
            {:else}
              Refresh Layer Token
            {/if}
          </Cell>

          <Divider />

          <Cell
            onclick={() => {
              handleRemoveBot();
            }}
          >
            {#snippet before()}
              <DeleteIcon />
            {/snippet}
            {#if isDeletingBot}
              Removing...
            {:else}
              Remove Bot
            {/if}
          </Cell>
        </AccordionContent>
      </Accordion>
    </Section>
  {/if}
</List>

<style>
  .header--center-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 16px 0;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }

  .no-admins {
    color: var(--tgui--secondary_hint_color);
    text-align: center;
    padding: 8px 0;
  }

  .loading-text {
    color: var(--tgui--destructive_text_color);
  }
</style>
