<script lang="ts">
  import "telegram-ui/styles";
  import { platform } from "telegram-ui";
  import type { Page, Bot } from "./types";
  import PageMain from "./page_main.svelte";
  import { botsStore, loadBotsData, revokeAllAvatarObjectUrls } from "./store";

  import { onDestroy, onMount } from "svelte";

  import PageEdit from "./page_edit.svelte";
  import PageCreate from "./page_create.svelte";
  import PageChangeToken from "./page_change_token.svelte";
  import PageManageDonationButtons from "./page_manage_donation_buttons.svelte";
  import PageAddAdmin from "./page_add_admin.svelte";
  import PagePreviewStreamBot from "./page_preview_stream_bot.svelte";

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let currentPage: Page = $state("main");
  let isAppLoading = $state(true);
  let selectedBot: Bot | null = $state(null);

  function navigateTo(page: Page, bot?: Bot) {
    currentPage = page;
    if ((page === "edit" || page === "change_token") && bot) {
      selectedBot = bot;
    }
    if (app) {
      app.BackButton.isVisible = page !== "main";
    }
    window.scrollTo(0, 0);
  }

  onMount(() => {
    if (app) {
      app.expand();
      app.BackButton.onClick(() => {
        navigateTo("main");
      });
    }

    let isIOS = platform() === "ios";

    document.body.classList.add("wrapper");
    document.body.classList.add("wrapper--horizontal-limit");
    if (isIOS) {
      document.body.classList.add("wrapper-ios");
    }

    if (app) {
      loadBotsData(app.initData).then(() => {
        isAppLoading = false;
      });
    }

    return () => {
      document.body.classList.remove("wrapper");
      document.body.classList.remove("wrapper--horizontal-limit");
      if (isIOS) {
        document.body.classList.remove("wrapper-ios");
      }
    };
  });

  onDestroy(() => {
    revokeAllAvatarObjectUrls();
  });
</script>

<div>
  {#if currentPage === "main"}
    <PageMain {navigateTo} />
  {:else if currentPage === "edit"}
    <PageEdit {navigateTo} bot={selectedBot} />
  {:else if currentPage === "create"}
    <PageCreate {navigateTo} />
  {:else if currentPage === "change_token"}
    <PageChangeToken {navigateTo} bot={selectedBot} />
  {:else if currentPage === "manage_donation_buttons"}
    <PageManageDonationButtons {navigateTo} bot={selectedBot} />
  {:else if currentPage === "preview_stream_bot"}
    <PagePreviewStreamBot {navigateTo} bot={selectedBot} />
  {:else if currentPage === "add_admin"}
    <PageAddAdmin {navigateTo} bot={selectedBot} />
  {/if}
</div>

<style>
  :global(body) {
    user-select: none;
  }

  .app-loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    width: 100%;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    border-top-color: var(--tg-theme-button-color, #50a8eb);
    animation: spin 1s ease-in-out infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
