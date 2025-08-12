<script lang="ts">
  import type { Page, Bot } from "./types";
  import { onDestroy, onMount } from "svelte";
  import {
    Button,
    Divider,
    Title,
    Caption,
    Section,
    SectionFooter,
    List,
    Cell,
    DeleteIcon,
    SectionHeader,
    PremiumStarIcon,
    Image,
  } from "telegram-ui";
  import { addDonationButtonToBot, botsStore } from "./store";
  import "telegram-ui/styles";
  import { type DonationButton } from "../stream_bot/types";
  import { getConfig, updateConfig, makeTestDonation } from "./queries";
  import { get } from "svelte/store";

  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | undefined } =
    $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let preview_data = $state(
    get(botsStore).data?.bots.find((b) => b.id === bot?.id)?.preview_data
  );
  let hasChanges = $state(false);
  let isUpdating = $state(false);

  onMount(() => {
    if (!bot) {
      navigateTo("main");
    }

    // Update preview_data from bot if it has been modified
    // if (bot?.preview_data) {
    //   preview_data = bot.preview_data;
    //   hasChanges = true;
    // }

    if (app) {
      app.BackButton.onClick(() => {
        navigateTo("edit", bot || undefined);
      });

      if (!preview_data) {
        getConfig(app.initData, bot!.id).then((res) => {
          if (res.success) {
            res.data.donation_buttons.forEach((button, ind) => {
              button.id = ind;
            });
            preview_data = res.data;
            addDonationButtonToBot(bot!.id, preview_data.donation_buttons);
          } else {
            console.log("error", res.error);
          }
        });
      }
    }
  });

  const handleDeleteButton = (id: number) => {
    if (!preview_data) return;
    preview_data.donation_buttons = preview_data.donation_buttons.filter(
      (button) => button.id !== id
    );

    preview_data.donation_buttons.forEach((button, index) => {
      button.id = index;
    });

    bot!.preview_data = preview_data;

    handleUpdateConfig();
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

  const handleTestDonation = () => {
    if (!bot) return;

    makeTestDonation(
      app!.initData,
      bot.id,
      100,
      "https://i.imgur.com/892vhef.jpeg"
    ).then((res) => {
      console.log("test donation res", res);
      if (res.success) {
        console.log("test donation successed sent");
      } else {
        //todo show error
        console.log("test donation error", res.error);
      }
    });
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

  <!-- <Button
    mode="filled"
    size="m"
    onclick={handleUpdateConfig}
    loading={isUpdating}
    disabled={!hasChanges}
  >
    Confirm
  </Button> -->

  <Button mode="filled" size="m" onclick={handleTestDonation}>
    Test Donation
  </Button>

  <Section>
    {#snippet header()}
      <div class="header-row">
        <SectionHeader>Current Donation Buttons</SectionHeader>
        <Button
          mode="plain"
          size="s"
          onclick={() => {
            navigateTo("add_donation_button", bot || undefined);
          }}>Add</Button
        >
      </div>
    {/snippet}
    <List>
      {#if preview_data && preview_data.donation_buttons.length > 0}
        {#each preview_data.donation_buttons as button (button.id)}
          <Cell style="padding: 0px;">
            {#snippet before()}
              <Image src={button.source_url} size={48} />
            {/snippet}

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
    <Caption weight={3} level={1}>
      Donation buttons allow your viewers to support your bot with predefined
      amounts.
      <br />
      You can add up to 10 donation buttons with different amounts and labels.
    </Caption>
  </SectionFooter>
</List>

<style>
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
</style>
