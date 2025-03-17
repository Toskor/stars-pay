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

</script>


<div class="content">
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

