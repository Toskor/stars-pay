<script lang="ts">
  import { onMount } from "svelte";
  import type { AppConfig } from "./tg_types";
  import TelegramAppSettings from "./telegram_app_settings.svelte";

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let app_config_json = '{"json_to_replace":""}';

  //config for tests
  let app_config_json1 = `{
   "header_text":"Yoml | Best stream app",
   "buttons":[],
   "api_url":"https://advanced-oddly-herring.ngrok-free.app/",
   "page_description": "Here you can make star donation for Streamer ",
   "owner": 348135868,
   "admins": [348135868]
}
`;
  let app_config: AppConfig = $state(JSON.parse(app_config_json));

  let test_str: string = app_config_json;

  let isSettingsButtonEnabled = $state(false);
  async function checkUserAccess() {
    //todo remove initDataUnsafe.user.id is undef
    setTimeout(() => {}, 100);

    isSettingsButtonEnabled = app_config.admins.includes(
      app.initDataUnsafe.user.id
    );
  }

  let showSettings = $state(false);
  function toggleSettings() {
    showSettings = !showSettings;
  }

  onMount(() => {
    app.expand();
    checkUserAccess();
    // isSettingsButtonEnabled = true;
    console.log(app.initData);
  });
</script>

<div class="fixed-header">
  <span class="header-text">{app_config.header_text}</span>
</div>

<button
  class="settings-button {isSettingsButtonEnabled ? '' : 'hidden'}"
  onclick={toggleSettings}
>
  <img
    src="https://cdn-icons-png.flaticon.com/512/10233/10233697.png"
    alt="Settings"
  />
</button>

{#if showSettings}
  <TelegramAppSettings
    {app_config}
    user_id={app.initDataUnsafe.user.id}
    init_data={app.initData}
  ></TelegramAppSettings>
{/if}

<div class="content">
  <!-- {test_str} -->
  <span>{app_config.page_description}</span>

  <hr class="divider" />

  <div class="button-grid">
    {#each app_config.buttons as button}
      <button
        class="donate-button"
        onclick={() =>
          app.openInvoice(button.invoice_url, (status) => {
            if (status === "paid") {
              // animation "success donation"? telegram already shows that
              // show that donation success with message and image
            }
          })}>{button.name}</button
      >
    {:else}
      <span class="text-context"
        >No donaiton options here. Add them in the settings.</span
      >
    {/each}
  </div>
</div>

<style>
  .settings-button {
    position: fixed;
    top: 5px;
    right: 2px;
    background: none;
    border: none;
    cursor: pointer;
    z-index: 1001;
  }
  .settings-button img {
    width: 30px;
    height: 30px;
  }
  .fixed-header {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    background-color: #f8f9fa;
    padding: 10px;
    text-align: center;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    z-index: 1000;
  }
  .content {
    margin-top: 50px;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .button-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    max-width: 600px;
  }
  .donate-button {
    flex: 1 1 calc(33.333% - 10px);
    height: 100px;
    box-sizing: border-box;
  }
  .divider {
    width: 100%;
    border: none;
    border-top: 1px solid #ccc;
    margin: 20px 0;
  }
  .hidden {
    display: none;
  }
</style>
