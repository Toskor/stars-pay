<script lang="ts">
  import "telegram-ui/styles";
  import type { Page } from "./types";
  import PageMain from "./page_main.svelte";

  import { onMount } from "svelte";

  import PageEdit from "./page_edit.svelte";
  import PageCreate from "./page_create.svelte";

  const api_url =
    "https://advanced-oddly-herring.ngrok-free.app/stardonationservice/";

  //todo remove safety check, add just for testing
  let app =
    //@ts-ignore
    (typeof window !== "undefined" && window.Telegram?.WebApp) || undefined;

  // let controlled_bots: ControlledBots | null = $state({
  //   bots: [
  //     {
  //       id: "stardonation",
  //       controll_type: "owner",
  //       username: "StarDonationBot",
  //       owner: {
  //         id: 1,
  //         username: "Torsor",
  //         avatar_url:
  //           "https://avatars.mds.yandex.net/i?id=c9ceb9a07ba909fe17c4eeb9dd83dfb4_l-12184992-images-thumbs&n=13",
  //       },
  //       admins: [
  //         {
  //           id: 1,
  //           username: "Torsor",
  //           avatar_url:
  //             "https://avatars.mds.yandex.net/i?id=c9ceb9a07ba909fe17c4eeb9dd83dfb4_l-12184992-images-thumbs&n=13",
  //         },
  //       ],
  //       avatar_url:
  //         "https://lastfm.freetls.fastly.net/i/u/ar0/0a087701e16a6f89cf98f0242dcdb3e8.png",
  //     },
  //   ],
  // });

  // async function getControlledBots(): Promise<ControlledBots | null> {
  //   let res = await fetch(`${api_url}getControlledBots`, {
  //     method: "GET",
  //     headers: {
  //       "Content-Type": "application/json;charset=utf-8",
  //       "X-Telegram-InitData": app.initData,
  //       // todo remove
  //       "ngrok-skip-browser-warning": "",
  //     },
  //   });

  //   if (res.ok) {
  //     let json: ControlledBots = await res.json();
  //     return json;
  //   } else {
  //     let err_text = await res.text();
  //     console.error(err_text);
  //     return null;
  //   }
  // }

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

  let isIOS = $state(true);

  let currentPage: Page = $state("main");
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
<!-- <div class="container">
  <Modal bind:showModal={show_add_bot_modal}>
    {#snippet header()}
      <h3 class="text-lg font-bold mb-4">Add Bot</h3>
    {/snippet}

    <div class="join">
      <button class="btn join-item rounded-r-full" onclick={handleAddBot}
        >Add</button
      >
    </div>

    <div class="join join-vertical">
      <button
        class="btn btn-primary btn-outline m-2"
        onclick={() => {
          app.openTelegramLink("https://t.me/botfather");
        }}>Go to BotFather</button
      >
    </div>
  </Modal> 
</div> -->

<style>
  :global(body) {
    user-select: none;
  }
</style>
