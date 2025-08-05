<script lang="ts">
  import type { Page, Bot } from "./types";
  import { onMount } from "svelte";
  import {
    Button,
    Title,
    Text,
    Image,
    Input,
    Section,
    SectionFooter,
    SectionHeader,
    List,
  } from "telegram-ui";
  import { type DonationButton, source_pool } from "../stream_bot/types";
  import {
    addDonationButtonToBot,
    botsStore,
    getDonationButtonsLen,
  } from "./store";
  import { updateConfig } from "./queries";
  import { get } from "svelte/store";

  let {
    navigateTo,
    bot,
  }: {
    navigateTo: (page: Page, bot?: Bot) => void;
    bot: Bot | undefined;
  } = $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  // Form state
  let name = $state("");
  let description = $state("");
  let amount = $state("");
  let selectedSourceId = $state(0);
  let isAdding = $state(false);
  let errorMessage = $state<string | null>(null);

  onMount(() => {
    if (!app) {
      navigateTo("main");
    }
    if (!bot) {
      navigateTo("main");
    }

    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("manage_donation_buttons", bot || undefined);
      });
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

    if (!bot) return;

    // Initialize preview_data if it doesn't exist
    if (!bot.preview_data) {
      bot.preview_data = {
        donation_buttons: [],
        title: "",
      };
    }

    const newButton: DonationButton = {
      id: getDonationButtonsLen(bot.id),
      name: name.trim(),
      description: description.trim(),
      amount: Number(amount),
      source_id: selectedSourceId,
      invoice_url: "",
    };

    addDonationButtonToBot(bot.id, [newButton]);

    let preview_data = get(botsStore).data?.bots.find(
      (b) => b.id === bot?.id
    )?.preview_data;

    updateConfig(app!.initData, bot!.id, JSON.stringify(preview_data)).then(
      (res) => {
        if (res.success) {
          app!.showPopup({
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

    // Reset form fields
    name = "";
    description = "";
    amount = "";
    selectedSourceId = 0;
    isAdding = false;

    // Navigate back
    navigateTo("manage_donation_buttons", bot);
  };
</script>

<List>
  <div class="layout-horizontal">
    <Title weight={1} level={3}>Add Donation Button</Title>
  </div>

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
          role="button"
          tabindex="0"
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              selectedSourceId = index;
            }
          }}
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
        navigateTo("manage_donation_buttons", bot || undefined);
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
</List>

<style>
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 16px;
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
