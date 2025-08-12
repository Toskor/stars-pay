<script lang="ts">
  import {
    Card,
    CardCell,
    Image,
    Button,
    platform,
    CardChip,
    Title,
    PremiumStarIcon,
  } from "telegram-ui";
  import { preview_default, source_pool, type PreviewData } from "./types";
  import { onDestroy, onMount } from "svelte";

  let { preview_data }: { preview_data?: PreviewData } = $props();

  let app_config_json = '{"json_to_replace":""}';

  let data = $state<PreviewData>({
    title: "Init title",
    donation_buttons: [],
  });

  let isPreview = $state(false);

  if (preview_data) {
    isPreview = true;
    data = preview_data;
  } else {
    try {
      data = JSON.parse(app_config_json) as PreviewData;
    } catch {
      console.error("Failed to parse app_config_json");
      data = {
        title: "Error pre view",
        donation_buttons: [],
      };
    }
  }

  //@ts-ignore
  let app = window.Telegram.WebApp;

  function openMainBotMiniApp() {
    if (app) {
      app.openTelegramLink(
        `https://t.me/StarDonationServiceBot/app?startapp=start_param`
      );
    }
  }

  function openInvoice(invoice_url: string) {
    if (app && !isPreview) {
      app.openInvoice(invoice_url, (status: string) => {
        if (status === "paid") {
          // animation "success donation"? telegram already shows that
        }
      });
    }
  }

  onMount(() => {
    if (app) {
      app.expand();

      console.log("stream bot app data", data);

      //main button
      app.MainButton.isVisible = true;
      app.MainButton.text = "Refresh";
      app.MainButton.onClick(() => {
        location.reload();
      });

      //secondary button
      app.SecondaryButton.isVisible = true;
      app.SecondaryButton.text = "Mini App Settings";
      app.SecondaryButton.onClick(() => {
        openMainBotMiniApp();
      });
    }

    let isIOS = platform() === "ios";

    document.body.classList.add("wrapper");
    document.body.classList.add("wrapper--horizontal-limit");
    if (isIOS) {
      document.body.classList.add("wrapper-ios");
    }

    // return () => {
    //   document.body.classList.remove("wrapper");
    //   document.body.classList.remove("wrapper--horizontal-limit");
    //   if (isIOS) {
    //     document.body.classList.remove("wrapper-ios");
    //   }
    // };
  });

  onDestroy(() => {
    if (app) {
      app.MainButton.isVisible = false;
      app.SecondaryButton.isVisible = false;
    }
  });
</script>

<Title weight={2} level={1}>
  {data.title}
</Title>

<div class="horizontal-scroll">
  {#each data.donation_buttons as button}
    <Card
      style="width: 254px; height: 308px; flex: 0 0 auto; margin-right: 12px;"
    >
      <CardChip
        onclick={() => {
          openInvoice(button.invoice_url);
        }}
      >
        {button.amount}
        <PremiumStarIcon />
      </CardChip>

      <Image
        alt={button.name}
        src={button.source_url}
        style="width: 100%; height: 230px; object-fit: cover; display: block;"
      />
      <CardCell
        onclick={() => {
          console.log("card clicked");
          openInvoice(button.invoice_url);
        }}
      >
        {button.name}
        {#snippet subtitle()}
          {button.description}
        {/snippet}
      </CardCell>
    </Card>
  {/each}
</div>

<Button onclick={openMainBotMiniApp} style="margin: 16px;">
  Open main bot mini app
</Button>

<style>
  .horizontal-scroll {
    display: flex;
    overflow-x: auto;
    flex-wrap: nowrap;
    gap: 0;
    padding: 8px 0;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    scroll-snap-type: x mandatory;
  }

  :global {
    .horizontal-scroll > * {
      scroll-snap-align: start;
    }
  }

  .horizontal-scroll::-webkit-scrollbar {
    display: none;
  }
</style>
