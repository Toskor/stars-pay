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


