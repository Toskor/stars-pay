<script lang="ts">
  import { Card, CardCell, Image, Button, platform } from "telegram-ui";
  import { preview_default, source_pool, type PreviewData } from "./types";
  import { onMount } from "svelte";

  let app_config_json = '{"json_to_replace":""}';
  let preview_data = $state<PreviewData>(preview_default);

  //@ts-ignore
  let app = window.Telegram.WebApp;

  function openMainBotMiniApp() {
    if (app) {
      app.openTelegramLink(
        `https://t.me/StarDonationServiceBot/app?startapp=start_param`
      );
    }
  }

  onMount(() => {
    if (app) {
      app.expand();
    }

    let isIOS = platform() === "ios";

    document.body.classList.add("wrapper");
    document.body.classList.add("wrapper--horizontal-limit");
    if (isIOS) {
      document.body.classList.add("wrapper-ios");
    }

    return () => {
      document.body.classList.remove("wrapper");
      document.body.classList.remove("wrapper--horizontal-limit");
      if (isIOS) {
        document.body.classList.remove("wrapper-ios");
      }
    };
  });
</script>

<div class="horizontal-scroll">
  {#each preview_data.donation_buttons as button}
    <Card
      style="width: 254px; height: 308px; flex: 0 0 auto; margin-right: 12px;"
    >
      <Image
        alt={button.name}
        src={source_pool[button.source_id]}
        style="width: 100%; height: 230px; object-fit: cover; display: block;"
      />
      <CardCell
        onclick={() => {
          console.log("card clicked");
          if (app) {
            app.openInvoice(button.invoice_url, (status: string) => {
              if (status === "paid") {
                // animation "success donation"? telegram already shows that
              }
            });
          }
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
  }

  .horizontal-scroll::-webkit-scrollbar {
    display: none;
  }
</style>
