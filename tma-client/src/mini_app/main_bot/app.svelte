<script lang="ts">
  import "telegram-ui/styles";
  import { platform } from "telegram-ui";
  import type { Page } from "./types";
  import PageMain from "./page_main.svelte";
  import { botsStore, loadBotsData } from "./store";

  import { onMount } from "svelte";

  import PageEdit from "./page_edit.svelte";
  import PageCreate from "./page_create.svelte";

  //@ts-ignore
  let app = window.Telegram.WebApp;

  // async function addBotQuery(bot_token: string) {
  //   let res = await fetch(`${api_url}addBot`, {
  //     method: "POST",
  //     headers: {
  //       "Content-Type": "application/json;charset=utf-8",
  //       "X-Telegram-InitData": app.initData,
  //       // todo remove
  //       "ngrok-skip-browser-warning": "",
  //     },
  //     body: JSON.stringify({ bot_token }),
  //   });

  //   if (res.ok) {
  //     alert("Bot added");
  //   } else {
  //     let err = await res.text();
  //     console.error(err);
  //     alert("Error: " + err);
  //   }
  // }

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

    // Загружаем данные ботов при инициализации приложения
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

<!-- helpfull code -->
<!--<div class="join join-vertical">
      <button
        class="btn btn-primary btn-outline m-2"
        onclick={() => {
          app.openTelegramLink("https://t.me/botfather");
        }}>Go to BotFather</button
      >
    </div>
 -->

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
