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
    List,
    Cell,
    DeleteIcon,
    SectionHeader,
    PremiumStarIcon,
  } from "telegram-ui";
  import { botsStore } from "./store";
  import "telegram-ui/styles";
  import { preview_default, type DonationButton } from "../stream_bot/types";
  import { getConfig, updateConfig } from "./queries";

  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | undefined } =
    $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let preview_data = $state(bot?.preview_data);
  let newButtonAmount = $state("");
  let newButtonLabel = $state("");
  let isAddingButton = $state(false);
  let errorMessage = $state<string | null>(null);
  // Add a state variable to track if there are unsaved changes
  let hasChanges = $state(false);
  // Add a state variable to track if the update is in progress
  let isUpdating = $state(false);

  onMount(() => {
    if (!bot) {
      navigateTo("main");
    }

    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("edit", bot || undefined);
      });

      if (!preview_data) {
        //todo query for preview data
        getConfig(app.initData, bot!.id).then((res) => {
          if (res.success) {
            res.data.donation_buttons.forEach((button, ind) => {
              button.id = ind;
            });
            preview_data = res.data;
          } else {
            console.log("error", res.error);
          }
        });
      }
    }
  });

  const handleAddButton = () => {
    if (!preview_data) return;
    if (isAddingButton) return;

    // Validate inputs
    // if (!newButtonAmount.trim() || isNaN(Number(newButtonAmount))) {
    //   errorMessage = "Please enter a valid amount";
    //   return;
    // }

    // if (!newButtonLabel.trim()) {
    //   errorMessage = "Please enter a button label";
    //   return;
    // }

    isAddingButton = true;
    errorMessage = null;

    const newButton: DonationButton = {
      id: preview_data.donation_buttons.length,
      amount: 1000,
      name: "New Donation Button",
      description: "",
      source_id: 0,
      invoice_url: "",
    };

    preview_data.donation_buttons = [
      ...preview_data.donation_buttons,
      newButton,
    ];

    // Reset form
    newButtonAmount = "";
    newButtonLabel = "";
    isAddingButton = false;

    // Mark as changed
    hasChanges = true;

    // Show success message
    if (app) {
      app.showPopup({
        title: "Success",
        message: "Donation button added successfully!",
        buttons: [{ type: "ok" }],
      });
    }
  };

  const handleDeleteButton = (id: number) => {
    if (!preview_data) return;
    preview_data.donation_buttons = preview_data.donation_buttons.filter(
      (button) => button.id !== id
    );

    preview_data.donation_buttons.forEach((button, index) => {
      button.id = index;
    });

    bot!.preview_data = preview_data;

    // Mark as changed
    hasChanges = true;
  };

  const handleUpdateConfig = () => {
    isUpdating = true;
    bot!.preview_data = preview_data;

    if (app) {
      updateConfig(app.initData, bot!.id, JSON.stringify(preview_data)).then(
        (res) => {
          isUpdating = false;
          hasChanges = false;

          if (res.success) {
            app.showPopup({
              title: "Success",
              message: "Donation buttons updated successfully!",
              buttons: [{ type: "ok" }],
            });
          } else {
            //todo show error
            console.log("error", res.error);
          }
        }
      );
    }
  };
</script>

<List>
  <div class="layout-horizontal">
    <Title weight={1} level={3}>Manage Donation Buttons</Title>
  </div>

  <Button
    mode="filled"
    size="m"
    onclick={() => {
      if (bot) {
        bot.preview_data = preview_data;
        navigateTo("preview_stream_bot", bot);
      }
    }}
  >
    Preview
  </Button>

  <Button
    mode="filled"
    size="m"
    onclick={handleUpdateConfig}
    loading={isUpdating}
    disabled={!hasChanges}
  >
    Confirm
  </Button>

  <Section>
    {#snippet header()}
      <div class="header-row">
        <SectionHeader>Current Donation Buttons</SectionHeader>
        <Button mode="plain" size="s" onclick={handleAddButton}>Add</Button>
      </div>
    {/snippet}
    <List>
      {#if preview_data && preview_data.donation_buttons.length > 0}
        {#each preview_data.donation_buttons as button (button.id)}
          <Cell>
            {button.name} - {button.amount}
            <PremiumStarIcon />

            {#snippet after()}
              <Button
                mode="destructive"
                size="s"
                onclick={() => handleDeleteButton(button.id)}
              >
                <DeleteIcon />
              </Button>
            {/snippet}
          </Cell>

          {#if button !== preview_data.donation_buttons[preview_data.donation_buttons.length - 1]}
            <Divider />
          {/if}
        {/each}
      {:else}
        <Cell>
          <div class="no-buttons">No donation buttons added yet</div>
        </Cell>
      {/if}
    </List>
  </Section>

  <SectionFooter centered={false}>
    <Text>
      Donation buttons allow your users to support your bot with predefined
      amounts.
      <br />
      You can add up to 10 donation buttons with different amounts and labels.
    </Text>
  </SectionFooter>
</List>

<style>
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 16px;
  }

  .no-buttons {
    color: var(--tgui--secondary_hint_color);
    text-align: center;
    padding: 8px 0;
  }
</style>
