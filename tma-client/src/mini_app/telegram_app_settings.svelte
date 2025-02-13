<script lang="ts">
  import { onMount } from "svelte";
  import type { AppConfig, Button } from "./tg_types";
  let {
    app_config,
    user_id: u_id,
    init_data,
  }: { app_config: AppConfig; user_id: number; init_data: string } = $props();

  let test_str = $state("");

  let title = $state("");
  let description = $state("");
  let payload = $state("");
  let amount = $state(0);

  let showAlert = $state(false);
  let alertMessage = $state("");

  let selectedButton = $state<Button | null>(null);

  async function query_create_invoice(
    user_id: number,
    title: string,
    description: string,
    payload: string,
    amount: number
  ): Promise<string> {
    test_str += "query start | ";
    let res = await fetch(`${app_config.api_url}createInvoice`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json;charset=utf-8",
        "X-Telegram-InitData": init_data,
        // todo remove
        "ngrok-skip-browser-warning": "",
      },
      body: JSON.stringify({ user_id, title, description, payload, amount }),
    });

    if (res.ok) {
      let json = await res.json();
      return json.invoice_url;
    } else {
      test_str +=
        "query not ok | url " + `${app_config.api_url}createInvoice` + " | ";
      let t = await res.text();
      test_str += t + " | ";
      //todo show error
      let json = await res.json();
      test_str = json.description;
      return "";
    }
  }

  async function createDonationOption() {
    const invoice_url = await query_create_invoice(
      u_id,
      title,
      description,
      payload,
      amount
    );

    if (invoice_url) {
      app_config.buttons.push({
        name: title,
        invoice_url: invoice_url,
      });
    }

    title = "";
    description = "";
    payload = "";
    amount = 0;

    await sendConfigToServer();
  }

  async function sendConfigToServer() {
    const res = await fetch(`${app_config.api_url}updateConfig`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json;charset=utf-8",
        "X-Telegram-InitData": init_data,
        // todo remove
        "ngrok-skip-browser-warning": "",
      },
      body: JSON.stringify({ app_config: JSON.stringify(app_config) }),
    });

    if (res.ok) {
      alertMessage = "Success! ";
      showAlert = true;
    } else {
      let json = await res.json();
      alertMessage = json.description;
      showAlert = true;
    }
  }

  function selectButton(button: Button) {
    selectedButton = button;
  }

  async function removeSelectedButton() {
    if (selectedButton) {
      app_config.buttons = app_config.buttons.filter(
        (button) => button.invoice_url !== selectedButton?.invoice_url
      );
      selectedButton = null;
      await sendConfigToServer();
    }
  }
</script>

<div class="settings-model">
  <span>Create donation option</span>
  <br />

  <input type="text" placeholder="Title" bind:value={title} />
  <input type="text" placeholder="Description" bind:value={description} />
  <input type="text" placeholder="Payload" bind:value={payload} />
  <input type="number" placeholder="Amount" bind:value={amount} />
  <br />

  <button onclick={createDonationOption}>Create</button>
  <br />

  <select bind:value={selectedButton}>
    <option value={null}>-- Select --</option>
    {#each app_config.buttons as button, index}
      <option value={button} id={"button-" + index}>{button.name}</option>
    {/each}
  </select>

  <button onclick={removeSelectedButton}> Delete </button>

  <br />
  <span>{test_str}</span>

  {#if showAlert}
    <div class="alert">
      {alertMessage}
      <button onclick={() => (showAlert = false)}>Close</button>
    </div>
  {/if}
</div>

<style>
  .settings-model {
    position: fixed;
    top: 15%;
    left: 5%;
    right: 5%;
    bottom: 10%;
    background-color: rgb(141, 136, 136);
    padding: 20px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    z-index: 1002;
    border-radius: 10px;
  }
  .alert {
    position: fixed;
    top: 20%;
    left: 50%;
    transform: translateX(-50%);
    background-color: rgb(234, 110, 110);
    padding: 10px;
    border: 1px solid black;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    z-index: 1003;
    border-radius: 5px;
  }
</style>
