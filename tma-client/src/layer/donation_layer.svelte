<script lang="ts">
  import { onMount } from "svelte";
  import type { LayerConfig } from "./types";
  import { WSClient, type WSMessage } from "./ws";
  import { donationStore } from "./donation_store";
  /* 
  This layer is used to display a donation image.
  It connects to a websocket and displays the image.
  The image is displayed for 5 seconds 
    or if its gif until end of gif 
    or 5 seconds repeat of gif 
    and then hidden.
  need queue if get donation event while image is displaying
  some control buttons:
  button stop/start display donations
  need to show current queue size
  clean queue button and when close layer


*/

  let layer_json_config =
    '{"ws_url": "wss://advanced-oddly-herring.ngrok-free.app/ws/star_donation_bot?ws_token=782cecc0-0e17-42f6-8625-4361c92ab553"}';
  let test_gif_url = "https://i.giphy.com/media/3oEjI6SIIHBdRx6PBI/giphy.gif";
  let test_image_url =
    "https://avatars.mds.yandex.net/i?id=3ef58cad5f77fcebe674582d17765372_l-4032453-images-thumbs&n=13";

  let layer_config: LayerConfig = JSON.parse(layer_json_config);
  let wsClient: WSClient;
  let currentImage: string | null = null;
  let showImage = false;
  let displayTimeout: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    console.log("Donation layer mounted");
    wsClient = new WSClient({
      ws_url: layer_config.ws_url,
    });

    wsClient.onMessage((message) => {
      if (message.ok) {
        donationStore.addToQueue(message);
      } else {
        console.error("Error:", message.error);
      }
    });

    wsClient.connect().catch((error) => {
      console.error("Failed to connect to WebSocket:", error);
    });

    return () => {
      wsClient.disconnect();
      donationStore.clearQueue();
      if (displayTimeout) {
        clearTimeout(displayTimeout);
      }
    };
  });

  function handleToggleDisplay() {
    donationStore.toggleDisplay();
  }

  function handleClearQueue() {
    donationStore.clearQueue();
  }

  $: if ($donationStore.queue.length > 0 && $donationStore.isDisplayEnabled) {
    const nextItem = $donationStore.queue[0];
    currentImage = nextItem.invoice_payload;
    showImage = true;

    if (displayTimeout) {
      clearTimeout(displayTimeout);
    }

    if (nextItem.isGif) {
      // For GIFs, we'll show them for 5 seconds or until they finish
      displayTimeout = setTimeout(() => {
        showImage = false;
        donationStore.removeFromQueue(nextItem.id);
      }, 5000);
    } else {
      // For static images, show for 5 seconds
      displayTimeout = setTimeout(() => {
        showImage = false;
        donationStore.removeFromQueue(nextItem.id);
      }, 5000);
    }
  }
</script>

<div class="controls">
  <button onclick={handleToggleDisplay}>
    {$donationStore.isDisplayEnabled ? "Stop Display" : "Start Display"}
  </button>
  <button onclick={handleClearQueue}>Clear Queue</button>
  <button
    onclick={() => {
      donationStore.addToQueue({
        ok: true,
        from: "username",
        total_amount: 100,
        invoice_payload: test_image_url,
      });
    }}>Add test item</button
  >
  <span class="queue-size">Queue size: {$donationStore.queue.length}</span>
</div>

<div class="image-container" class:hidden={!showImage}>
  {#if currentImage}
    {console.log("image url", currentImage)}
    <img src={currentImage} alt={`Donation source`} />
  {/if}
</div>

<style>
  .controls {
    position: fixed;
    top: 10px;
    right: 10px;
    z-index: 1001;
    background: rgba(0, 0, 0, 0.7);
    padding: 10px;
    border-radius: 5px;
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .controls button {
    padding: 5px 10px;
    border-radius: 3px;
    border: none;
    background: #4a4a4a;
    color: white;
    cursor: pointer;
  }

  .controls button:hover {
    background: #5a5a5a;
  }

  .queue-size {
    color: white;
    font-size: 14px;
  }

  .image-container {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 1000;
    transition: opacity 0.5s ease-in-out;
  }

  .image-container img {
    max-width: 90vw;
    max-height: 90vh;
    object-fit: contain;
  }

  .hidden {
    opacity: 0;
    pointer-events: none;
  }
</style>
