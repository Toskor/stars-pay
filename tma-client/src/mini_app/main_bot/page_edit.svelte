<script lang="ts">
  import { onMount } from "svelte";

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
  } from "telegram-ui";
  import type { Page, Bot } from "./types";
  import { removeBotAdmin, removeBot } from "./queries";
  import { botsStore, refreshBotsData } from "./store";
  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | null } =
    $props();

  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");
  let isRemoving = $state<number | null>(null);
  let isDeletingBot = $state<boolean>(false);

  //@ts-ignore
  let app = window.Telegram.WebApp;

  onMount(() => {
    if (!bot) {
      navigateTo("main");
    }
  });

  async function removeAdmin(adminId: number, adminName: string) {
    if (!app || !bot) return;

    isRemoving = adminId;
    try {
      const response = await removeBotAdmin(app.initData, bot.id, adminId);

      if (response.success) {
        // Update the store by removing the admin from the current bot
        botsStore.update((store) => {
          if (store.data && store.data.bots) {
            const updatedBots = store.data.bots.map((storeBot) => {
              if (storeBot.id === bot.id) {
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

        // Update the local bot object to reflect changes in UI
        if (bot) {
          bot.admins = bot.admins.filter((a) => a.id !== adminId);
        }

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
    if (!app || !bot) return;

    // Show confirmation popup
    app.showConfirm(
      `Are you sure you want to remove bot "${bot.name}"? This action cannot be undone.`,
      async (confirmed: boolean) => {
        if (confirmed) {
          try {
            isDeletingBot = true;
            const response = await removeBot(app.initData, bot.id);

            if (response.success) {
              // Update the store by removing the bot
              botsStore.update((store) => {
                if (store.data && store.data.bots) {
                  return {
                    ...store,
                    data: {
                      ...store.data,
                      bots: store.data.bots.filter((b) => b.id !== bot.id),
                    },
                  };
                }
                return store;
              });

              // Show success message
              app.showPopup({
                title: "Success",
                message: `Bot ${bot.name} removed successfully`,
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
</script>

<List>
  <div class="header--center-container">
    <Avatar size={128} src={bot?.avatar || ""} acronym={bot?.name?.[0] || ""} />
  </div>

  <Section header="Owner">
    <Cell>
      {#snippet before()}
        <Avatar
          size={48}
          src={bot?.owner?.avatarUrl || ""}
          acronym={bot?.owner?.name?.[0] || ""}
        />
      {/snippet}

      {#snippet children()}
        {bot?.owner?.name || ""}
      {/snippet}

      {#snippet subtitle()}
        {bot?.owner?.username || ""}
      {/snippet}
    </Cell>
  </Section>

  <Section>
    {#snippet header()}
      <div class="header-row">
        <SectionHeader>Bot admins</SectionHeader>
        <Button mode="plain" size="s">Add</Button>
      </div>
    {/snippet}

    {#if bot?.admins && bot.admins.length > 0}
      {#each bot.admins as admin, index}
        <Cell>
          {#snippet before()}
            <Avatar
              size={48}
              src={admin.avatarUrl || ""}
              acronym={admin.name?.[0] || ""}
            />
          {/snippet}

          {#snippet after()}
            {#if bot?.userRole === "owner"}
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

        {#if index < bot.admins.length - 1}
          <Divider />
        {/if}
      {/each}
    {:else}
      <Cell>
        <div class="no-admins">No admins added yet</div>
      </Cell>
    {/if}
  </Section>

  {#if bot?.userRole === "owner"}
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
              navigateTo("change_token", bot);
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
    justify-content: center;
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
