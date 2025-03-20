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
  import {
    preview_default,
    type DonationButton,
    source_pool,
  } from "../stream_bot/types";
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

  let hasChanges = $state(false);
  let isUpdating = $state(false);

  // Add button form state
  let showAddForm = $state(false);
  let name = $state("");
  let description = $state("");
  let amount = $state("");
  let selectedSourceId = $state(0);
  let isAdding = $state(false);

  onMount(() => {
    if (!bot) {
      navigateTo("main");
    }

    if (app) {
      app.BackButton.onClick(() => {
        if (showAddForm) {
          showAddForm = false;
        } else {
          navigateTo("edit", bot || undefined);
        }
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
    if (!name.trim()) {
      errorMessage = "Please enter a name for the donation button";
      return;
    }

    if (!amount.trim() || isNaN(Number(amount))) {
      errorMessage = "Please enter a valid amount";
      return;
    }

    isAdding = true;
    errorMessage = null;

    if (!bot || !preview_data) return;

    const newButton: DonationButton = {
      id: preview_data.donation_buttons.length,
      name: name.trim(),
      description: description.trim(),
      amount: Number(amount),
      source_id: selectedSourceId,
      invoice_url: "",
    };

    preview_data.donation_buttons = [
      ...preview_data.donation_buttons,
      newButton,
    ];

    // Reset form fields
    name = "";
    description = "";
    amount = "";
    selectedSourceId = 0;
    isAdding = false;

    // Hide the form
    showAddForm = false;

    hasChanges = true;
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
    <Title weight={1} level={3}
      >{showAddForm ? "Add Donation Button" : "Manage Donation Buttons"}</Title
    >
  </div>

  {#if !showAddForm}
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
          <Button
            mode="plain"
            size="s"
            onclick={() => {
              showAddForm = true;
            }}>Add</Button
          >
        </div>
      {/snippet}
      <List>
        {#if preview_data && preview_data.donation_buttons.length > 0}
          {#each preview_data.donation_buttons as button (button.id)}
            <Cell>
              {button.name}

              {#snippet subhead()}
                {button.amount}
                <PremiumStarIcon />
              {/snippet}

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
  {:else}
    <!-- Add Donation Button Form -->
    <Section>
      <SectionHeader>Button Information</SectionHeader>

      {#if errorMessage}
        <div class="error-message">{errorMessage}</div>
      {/if}

      <div class="form-group">
        <Input placeholder="Name" bind:value={name} header="Name" />

        <Input
          placeholder="Description (optional)"
          bind:value={description}
          header="Description"
        />

        <Input placeholder="Amount" bind:value={amount} header="Amount" />
      </div>
    </Section>

    <Section>
      <SectionHeader>Select Source</SectionHeader>
      <div class="image-selector">
        {#each source_pool as source, index}
          <div
            class="image-option"
            class:selected={selectedSourceId === index}
            onclick={() => (selectedSourceId = index)}
          >
            <Image src={source} size={48} />
            <div
              class="radio-circle"
              class:selected={selectedSourceId === index}
            ></div>
          </div>
        {/each}
      </div>
    </Section>

    <div class="button-group">
      <Button
        mode="filled"
        size="m"
        stretched={true}
        onclick={handleAddButton}
        loading={isAdding}
      >
        Add Button
      </Button>
      <Button
        mode="plain"
        size="m"
        stretched={true}
        onclick={() => {
          showAddForm = false;
          errorMessage = null;
        }}
      >
        Cancel
      </Button>
    </div>

    <SectionFooter centered={false}>
      <Text>
        Configure your donation button with a name, description, amount, and
        select an image.
      </Text>
    </SectionFooter>
  {/if}
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

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }

  .error-message {
    color: var(--tgui--destructive_text_color);
    font-size: 14px;
    margin-bottom: 16px;
  }

  .image-selector {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-top: 8px;
  }

  .image-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    padding: 8px;
    border-radius: 8px;
  }

  .image-option.selected {
    background-color: var(--tgui--secondary_bg_color);
  }

  .radio-circle {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid var(--tgui--secondary_hint_color);
  }

  .radio-circle.selected {
    border-color: var(--tgui--button_color);
    background-color: var(--tgui--button_color);
    position: relative;
  }

  .radio-circle.selected::after {
    content: "";
    position: absolute;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background-color: white;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

  .button-group {
    margin: 24px 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
