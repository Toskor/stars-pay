<script lang="ts">
  import { onMount } from "svelte";
  import Goal from "./goal.svelte";
  import type { GoalProps } from "./types";
  import { WSClient, type WSMessage } from "../layer/ws";

  let propsJson = '{"json_to_replace":""}';
  let goalProps: GoalProps = $state(JSON.parse(propsJson));

  let wsUrl = "replace_with_ws_url";

  let wsClient: WSClient;
  function handleWsMessage(message: WSMessage) {
    if (message.ok) {
      if (message.type === "goalProps") {
        goalProps = message.props;
        console.log("success received goalProps", goalProps);
      } else if (message.type === "donation") {
        goalProps.progress += message.total_amount;
      }
    } else {
      console.error("Error:", message.error);
    }
  }

  onMount(() => {
    wsClient = new WSClient({
      ws_url: wsUrl,
    });

    wsClient.onMessage((message) => {
      handleWsMessage(message);
    });

    wsClient.connect().catch((error) => {
      // console.error("Failed to connect to WebSocket:", error);
    });

    return () => {
      wsClient.disconnect();
    };
  });
</script>

<Goal {goalProps} />

<style>
</style>
