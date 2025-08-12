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
    FileInput,
    Cell,
  } from "telegram-ui";
  import { type DonationButton, source_pool } from "../stream_bot/types";
  import {
    addDonationButtonToBot as addDonationButtonToBotStore,
    botsStore,
    getDonationButtonsLen,
  } from "./store";
  import { updateConfig, uploadImage } from "./queries";
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

  // Source selection state
  let sourceType = $state<"preloaded" | "upload">("preloaded");
  let image = $state<File | undefined>(undefined);

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

  const handleAddButton = async () => {
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

    let source_url = "";
    if (sourceType === "upload") {
      if (!image) {
        errorMessage = "Please select an image file to upload";
        isAdding = false;
        return;
      }
      // upload image to server return url
      let res = await uploadImage(app!.initData, image);
      if (res.success) {
        source_url = res.data.image_url;
      } else {
        console.error("Error uploading image:", res.error);
        errorMessage = "Error uploading image. Please try again.";
        isAdding = false;
        return;
      }
    } else {
      // use preloaded source from source_pool
      source_url = source_pool[selectedSourceId];
    }
    image = undefined;

    // update newButton and store with url
    const newButton: DonationButton = {
      id: getDonationButtonsLen(bot.id),
      name: name.trim(),
      description: description.trim(),
      amount: Number(amount),
      source_url: source_url,
      invoice_url: "",
    };
    addDonationButtonToBotStore(bot.id, [newButton]);

    let preview_data = get(botsStore).data?.bots.find(
      (b) => b.id === bot?.id
    )?.preview_data;

    // update config request with newButton
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
    sourceType = "preloaded";
    isAdding = false;

    // Navigate back
    navigateTo("manage_donation_buttons", bot);
  };

  const handleImageChange = (e: Event) => {
    image = (e.target as HTMLInputElement).files?.[0];
    if (image) {
      sourceType = "upload";
    }
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

    <!-- Source type selector -->
    <div class="source-type-selector">
      <button
        class="source-type-button"
        class:active={sourceType === "preloaded"}
        onclick={() => {
          sourceType = "preloaded";
          image = undefined;
        }}
      >
        Preloaded sources
      </button>
      <button
        class="source-type-button"
        class:active={sourceType === "upload"}
        onclick={() => {
          sourceType = "upload";
        }}
      >
        Upload from device
      </button>
    </div>

    <!-- Preloaded images selector -->
    {#if sourceType === "preloaded"}
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
    {/if}

    <!-- File upload input -->
    {#if sourceType === "upload"}
      <div class="upload-section">
        <FileInput
          type="file"
          accept="image/png, image/jpeg, image/gif, image/apng"
          onchange={handleImageChange}
        >
          {#if image}
            <Cell>
              {#snippet subtitle()}
                {image!.size} bytes
              {/snippet}
              {#snippet children()}
                {image!.name}
              {/snippet}
            </Cell>
          {/if}
        </FileInput>
      </div>
    {/if}
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
</List>

<style>
  .source-type-selector {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    background-color: var(--tgui--secondary_bg_color);
    border-radius: 12px;
    padding: 4px;
    margin: 0px 5px;
  }

  .source-type-button {
    flex: 1;
    padding: 12px 16px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--tgui--text_color);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .source-type-button.active {
    background-color: var(--tgui--button_color);
    color: var(--tgui--button_text_color);
  }

  .source-type-button:hover:not(.active) {
    background-color: var(--tgui--hint_color);
  }

  .upload-section {
    margin-top: 12px;
  }

  .selected-file-info {
    margin-top: 12px;
    padding: 12px;
    background-color: var(--tgui--secondary_bg_color);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
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
    margin: 8px 5px 0px 5px;
  }

  .image-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    padding: 8px;
    margin-bottom: 5px;
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
