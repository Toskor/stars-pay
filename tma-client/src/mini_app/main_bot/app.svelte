<script lang="ts">
  import "telegram-ui/styles";
  import { platform } from "telegram-ui";
  import type { Page } from "./types";
  import PageMain from "./page_main.svelte";
  import { botsStore, loadBotsData, revokeAllAvatarObjectUrls } from "./store";

  import { onDestroy, onMount } from "svelte";

  import PageEdit from "./page_edit.svelte";
  import PageCreate from "./page_create.svelte";

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let currentPage: Page = $state("main");
  let isAppLoading = $state(true);

  function navigateTo(page: Page) {
    currentPage = page;
    app.BackButton.isVisible = page !== "main";
    window.scrollTo(0, 0);
  }

  onMount(() => {
    app.expand();
    app.BackButton.onClick(() => {
      navigateTo("main");
    });

    let isIOS = platform() === "ios";

    document.body.classList.add("wrapper");
    document.body.classList.add("wrapper--horizontal-limit");
    if (isIOS) {
      document.body.classList.add("wrapper-ios");
    }

    loadBotsData(app.initData).then(() => {
      isAppLoading = false;
    });

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
    <PageEdit {navigateTo} />
  {:else if currentPage === "create"}
    <PageCreate {navigateTo} />
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
