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
  } from "telegram-ui";
  import { botsStore } from "./store";
  import "telegram-ui/styles";

  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | null } =
    $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  // Sample donation buttons data - in a real app, this would come from an API
  let donationButtons = $state([
    { id: 1, amount: 5, currency: "USD", label: "Coffee" },
    { id: 2, amount: 10, currency: "USD", label: "Lunch" },
    { id: 3, amount: 20, currency: "USD", label: "Support" },
  ]);

  let newButtonAmount = $state("");
  let newButtonLabel = $state("");
  let newButtonCurrency = $state("USD");
  let isAddingButton = $state(false);
  let errorMessage = $state<string | null>(null);

  onMount(() => {
    console.log("manage donation buttons", bot);
    if (!bot) {
      navigateTo("main");
    }

    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("edit", bot || undefined);
      });
    }
  });

  const handleAddButton = () => {
    if (isAddingButton) return;

    // Validate inputs
    if (!newButtonAmount.trim() || isNaN(Number(newButtonAmount))) {
      errorMessage = "Please enter a valid amount";
      return;
    }

    if (!newButtonLabel.trim()) {
      errorMessage = "Please enter a button label";
      return;
    }

    isAddingButton = true;
    errorMessage = null;

    // In a real app, this would be an API call
    setTimeout(() => {
      // Add new button to the list
      const newButton = {
        id: Date.now(), // Generate a unique ID
        amount: Number(newButtonAmount),
        currency: newButtonCurrency,
        label: newButtonLabel,
      };

      donationButtons = [...donationButtons, newButton];

      // Reset form
      newButtonAmount = "";
      newButtonLabel = "";
      isAddingButton = false;

      // Show success message
      if (app) {
        app.showPopup({
          title: "Success",
          message: "Donation button added successfully!",
          buttons: [{ type: "ok" }],
        });
      }
    }, 500);
  };

  const handleDeleteButton = (id: number) => {
    donationButtons = donationButtons.filter((button) => button.id !== id);
  };
</script>

<List>
  <div class="layout-horizontal">
    <Title weight={1} level={3}>Manage Donation Buttons</Title>
  </div>

  <Section header="Current Donation Buttons">
    <List>
      {#if donationButtons.length > 0}
        {#each donationButtons as button}
          <Cell>
            {button.label} - {button.amount}
            {button.currency}

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

          {#if button !== donationButtons[donationButtons.length - 1]}
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
