<script lang="ts">
  import { onMount } from "svelte";
  import type { Bot, ControlledBots } from "./tg_types";
  import ControlledBotRepresentation from "./controlled_bot_representation.svelte";

  import { Button } from "telegram-ui";
  import { List } from "telegram-ui";

  import { Title } from "telegram-ui";
  import { Divider } from "telegram-ui";

  import "telegram-ui/styles";

  const api_url =
    "https://advanced-oddly-herring.ngrok-free.app/stardonationservice/";

  //@ts-ignore
  // let app = window.Telegram.WebApp;

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

  // let show_add_bot_modal = $state(false);
  // let bot_token = $state("");
  // function toggleAddForm() {
  //   show_add_bot_modal = !show_add_bot_modal;
  // }

  // function handleAddBot() {
  //   if (bot_token) {
  //     addBotQuery(bot_token);
  //     bot_token = "";
  //     show_add_bot_modal = false;
  //   }
  // }

  // let selected_bot: Bot | null = $state(null);
  // let show_bot_settings_modal = $state(false);

  let platform_name = $state("");
  let theme = $state("");
  let isIOS = $state(true);

  onMount(() => {
    document.body.classList.add("wrapper");
    // document.body.classList.add("wrapper--dark");
    document.body.classList.add("wrapper--horizontal-limit");
    if (isIOS) {
      document.body.classList.add("wrapper-ios");
    }

    return () => {
      document.body.classList.remove("wrapper");
      // document.body.classList.remove("wrapper--dark");
      document.body.classList.remove("wrapper--horizontal-limit");
      if (isIOS) {
        document.body.classList.remove("wrapper-ios");
      }
    };
  });
</script>

<div class="wrapper">
  platform_name: {platform_name}
  <br />
  theme: {theme}

  <Button
    onclick={() => {
      platform_name = "test";
      theme = "test";
    }}>upd</Button
  >
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
</style>
