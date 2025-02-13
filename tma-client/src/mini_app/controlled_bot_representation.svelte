<script lang="ts">
  import { onMount } from "svelte";

  let props: {
    bot_id: string;
    controll_type: "owner" | "admin";
    init_data: string;
    api_url: string;
  } = $props();

  async function add_admin(new_admin_id: string): Promise<void> {
    let res = await fetch(`${props.api_url}addBotAdmin`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json;charset=utf-8",
        "X-Telegram-InitData": props.init_data,
        // todo remove
        "ngrok-skip-browser-warning": "",
      },
      body: JSON.stringify({
        bot_id: props.bot_id,
        admin_id: new_admin_id,
      }),
    });

    if (res.ok) {
      alert("Admin added");
    } else {
      let err_text = await res.text();
      console.error(err_text);
      alert("Error: " + err_text);
    }
  }

  async function remove_admin(admin_id_for_remove: string): Promise<void> {
    let res = await fetch(`${props.api_url}removeBotAdmin`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json;charset=utf-8",
        "X-Telegram-InitData": props.init_data,
        // todo remove
        "ngrok-skip-browser-warning": "",
      },
      body: JSON.stringify({
        bot_id: props.bot_id,
        admin_id: admin_id_for_remove,
      }),
    });

    if (res.ok) {
      alert("Admin removed");
    } else {
      let err_text = await res.text();
      console.error(err_text);
      alert("Error: " + err_text);
    }
  }
</script>

<div class="section">
  <span>{props.controll_type} - {props.bot_id} </span>
  {#if props.controll_type === "owner"}
    <button onclick={() => add_admin(props.bot_id)}>
      Add admin
    </button>
    <button onclick={() => remove_admin(props.bot_id)}>
      Remove admin
    </button>
  {:else if props.controll_type === "admin"}
    <!-- <button onclick={() => admin_action(props.bot_id)}>
      Some admin button
    </button> -->
  {/if}
</div>

<style>
  .section {
    display: flex;
  }
</style>
