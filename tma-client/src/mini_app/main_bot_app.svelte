<script lang="ts">
  import { onMount } from "svelte";
  import type { Bot, ControlledBots } from "./tg_types";
  import ControlledBotRepresentation from "./controlled_bot_representation.svelte";
  import Modal from "./modal.svelte";

  const api_url =
    "https://advanced-oddly-herring.ngrok-free.app/stardonationservice/";
  //@ts-ignore
  let app = window.Telegram.WebApp;

  let controlled_bots: ControlledBots | null = $state({
    bots: [
      {
        id: "stardonation",
        controll_type: "owner",
        username: "StarDonationBot",
        owner: {
          id: 1,
          username: "Torsor",
          avatar_url:
            "https://avatars.mds.yandex.net/i?id=c9ceb9a07ba909fe17c4eeb9dd83dfb4_l-12184992-images-thumbs&n=13",
        },
        admins: [
          {
            id: 1,
            username: "Torsor",
            avatar_url:
              "https://avatars.mds.yandex.net/i?id=c9ceb9a07ba909fe17c4eeb9dd83dfb4_l-12184992-images-thumbs&n=13",
          },
        ],
        avatar_url:
          "https://lastfm.freetls.fastly.net/i/u/ar0/0a087701e16a6f89cf98f0242dcdb3e8.png",
      },
    ],
  });

  async function getControlledBots(): Promise<ControlledBots | null> {
    let res = await fetch(`${api_url}getControlledBots`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json;charset=utf-8",
        "X-Telegram-InitData": app.initData,
        // todo remove
        "ngrok-skip-browser-warning": "",
      },
    });

    if (res.ok) {
      let json: ControlledBots = await res.json();
      return json;
    } else {
      let err_text = await res.text();
      console.error(err_text);
      return null;
    }
  }

  async function addBotQuery(bot_token: string) {
    let res = await fetch(`${api_url}addBot`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json;charset=utf-8",
        "X-Telegram-InitData": app.initData,
        // todo remove
        "ngrok-skip-browser-warning": "",
      },
      body: JSON.stringify({ bot_token }),
    });

    if (res.ok) {
      alert("Bot added");
    } else {
      let err = await res.text();
      console.error(err);
      alert("Error: " + err);
    }
  }

  let show_add_bot_modal = $state(false);
  let bot_token = $state("");
  function toggleAddForm() {
    show_add_bot_modal = !show_add_bot_modal;
  }

  function handleAddBot() {
    if (bot_token) {
      addBotQuery(bot_token);
      bot_token = "";
      show_add_bot_modal = false;
    }
  }

  let selected_bot: Bot | null = $state(null);
  let show_bot_settings_modal = $state(false);
  // function openModal(bot) {
  //   selected_bot = bot;
  //   show_bot_settings_modal = true;
  // }

  onMount(async () => {
    controlled_bots = await getControlledBots();
    console.log(controlled_bots);
  });
</script>

<div class="container">
  {#if controlled_bots?.bots.length == 0}
    <span
      class="text-center text-lg font-semibold text-gray-600 block mt-4 mb-4"
    >
      You have no bots. Just add one!
    </span>
  {:else}
    <table class="table w-full m-4">
      <tbody>
        <!-- {#each controlled_bots.bots as bot (bot.id)}
          <tr>
            <td>
              <img
                src={bot.avatar_url}
                alt={bot.username}
                class="rounded-full w-12 h-12"
              />
            </td>
            <td>{bot.username}</td>
            <td>{bot.controll_type}</td>
            <td>
              <button class="btn btn-primary" onclick={() => openModal(bot)}
                >Settings</button
              >
            </td>
          </tr>
        {/each} -->
      </tbody>
    </table>
  {/if}

  <div class="divider"></div>

  <button class="btn btn-primary btn-block" onclick={toggleAddForm}
    >Add Bot</button
  >

  <Modal bind:showModal={show_add_bot_modal}>
    {#snippet header()}
      <h3 class="text-lg font-bold mb-4">Add Bot</h3>
    {/snippet}

    <div class="join">
      <input
        class="input input-bordered join-item"
        bind:value={bot_token}
        placeholder="Bot token"
      />
      <button class="btn join-item rounded-r-full" onclick={handleAddBot}
        >Add</button
      >
    </div>

    <div class="divider"></div>

    <div class="join join-vertical">
      <button
        class="btn btn-primary btn-outline m-2"
        onclick={() => {
          app.openTelegramLink("https://t.me/botfather");
        }}>Go to BotFather</button
      >

      <button class="btn btn-primary btn-outline m-2" onclick={() => {}}
        >Success add bot alert</button
      >
      <button class="btn btn-primary btn-outline m-2" onclick={() => {}}
        >Error add bot alert</button
      >
    </div>
  </Modal>

  <!-- <Modal bind:showModal={show_bot_settings_modal}>
    {#snippet header()}
      <h3 class="text-lg font-bold mb-4">Bot Settings</h3>
    {/snippet}
    {#if selected_bot}
      <div class="text-center">
        <img
          src={selected_bot.avatar_url}
          alt={selected_bot.username}
          class="rounded-full w-24 h-24 mx-auto"
        />
        <h4 class="text-xl font-semibold mt-2">{selected_bot.username}</h4>
      </div>

      <div class="divider"></div>

      <h5 class="text-lg font-bold mb-2">Administrators</h5>
      <ul>
        {#each selected_bot.admins as admin}
          <li class="flex items-center mb-2">
            <img
              src={admin.avatar_url}
              alt={admin.username}
              class="rounded-full w-8 h-8 mr-2"
            />
            <span>@{admin.username}</span>

            <details class="dropdown dropdown-end ml-auto">
              <summary class="btn m-1">⋮</summary>
              <ul
                class="menu dropdown-content bg-base-100 rounded-box z-[1] w-52 p-2 shadow"
              >
                <li>
                  <button
                    onclick={() => {
                      //todo
                    }}>Remove Admin</button
                  >
                </li>
              </ul>
            </details>
          </li>
        {/each}
      </ul>

      <div class="divider"></div>

      <div class="flex flex-col space-y-2">
        <button class="btn btn-primary" onclick={() => {}}> Add Admin </button>
        <button class="btn btn-secondary" onclick={() => {}}>
          Remove Admin
        </button>

        <button class="btn btn-info" onclick={() => {}}> Get invoice </button>
        <button class="btn btn-warning" onclick={() => {}}>
          Change Owner
        </button>
        <button class="btn btn-danger" onclick={() => {}}> Delete Bot </button>
      </div>
    {/if}
  </Modal> -->
</div>

<style>
</style>
