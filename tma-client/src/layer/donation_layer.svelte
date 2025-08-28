<script lang="ts">
  import { onMount } from "svelte";
  import type { LayerConfig } from "./types";
  import { WSClient, type WSMessage } from "./ws";
  import { donationStore } from "./donation_store";
  import { audioService } from "./audio_service";

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

  const PAUSE_BETWEEN_DONATIONS_MS = 1000; // 1 second pause between donations
  const DONATION_DISPLAY_DURATION_MS = 5000; // 5 seconds display duration
  let layer_json = `{"json_to_replace":""}`;
  console.log("layer_json", layer_json);

  let test_gif_url = "https://i.giphy.com/media/3oEjI6SIIHBdRx6PBI/giphy.gif";
  let test_image_url =
    "https://avatars.mds.yandex.net/i?id=3ef58cad5f77fcebe674582d17765372_l-4032453-images-thumbs&n=13";

  const testNames = ["Alex", "Maria", "John"];
  function getRandomTestDonation(): WSMessage {
    const randomName = testNames[Math.floor(Math.random() * testNames.length)];
    const randomStars = Math.floor(Math.random() * (1000 - 10 + 1)) + 10;
    return {
      ok: true,
      type: "donation",
      from: randomName,
      total_amount: randomStars,
      invoice_payload: test_image_url,
      message: "test message",
    };
  }

  let layer_config: LayerConfig = JSON.parse(layer_json);
  let wsClient: WSClient;
  let currentImage = $state<string | null>(null);
  let showImage = $state(false);
  let displayTimeout: ReturnType<typeof setTimeout> | null = null;
  let currentDonation = $state<{
    from: string;
    total_amount: number;
    message?: string;
  } | null>(null);
  let isFadingOut = $state(false);

  onMount(() => {
    audioService.init().then(() => {
      return audioService.preloadSounds(["donation_sound.mp3"]);
    });

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
      // console.error("Failed to connect to WebSocket:", error);
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

  $effect(() => {
    if (
      $donationStore.queue.length > 0 &&
      $donationStore.isDisplayEnabled &&
      !isFadingOut
    ) {
      const nextItem = $donationStore.queue[0];
      //just test sound
      audioService.playSound("donation_sound.mp3");
      currentImage = nextItem.invoice_payload;
      currentDonation = {
        from: nextItem.from,
        total_amount: nextItem.total_amount,
        message: nextItem.message,
      };
      showImage = true;

      if (displayTimeout) {
        clearTimeout(displayTimeout);
      }

      if (nextItem.isGif) {
        // For GIFs, we'll show them for 5 seconds or until they finish
        displayTimeout = setTimeout(() => {
          isFadingOut = true;
          showImage = false;
          // Wait for fade out animation to complete (0.5s) and pause
          setTimeout(() => {
            donationStore.removeFromQueue(nextItem.id);
            isFadingOut = false;
          }, 500 + PAUSE_BETWEEN_DONATIONS_MS);
        }, DONATION_DISPLAY_DURATION_MS);
      } else {
        // For static images, show for 5 seconds
        displayTimeout = setTimeout(() => {
          isFadingOut = true;
          showImage = false;
          // Wait for fade out animation to complete (0.5s) and pause
          setTimeout(() => {
            donationStore.removeFromQueue(nextItem.id);
            isFadingOut = false;
          }, 500 + PAUSE_BETWEEN_DONATIONS_MS);
        }, DONATION_DISPLAY_DURATION_MS);
      }
    }
  });
</script>

<div class="controls">
  <button onclick={handleToggleDisplay}>
    {$donationStore.isDisplayEnabled ? "Stop Display" : "Start Display"}
  </button>
  <button onclick={handleClearQueue}>Clear Queue</button>
  <button
    onclick={() => {
      donationStore.addToQueue(getRandomTestDonation());
    }}>Add test item</button
  >
  <span class="queue-size">Queue size: {$donationStore.queue.length}</span>
</div>

<div class="image-container" class:hidden={!showImage}>
  {#if currentImage}
    <div class="donation-content">
      <img src={currentImage} alt={`Donation source`} />

      {#if currentDonation}
        <div class="donation-main">
          <span class="donation-name">
            {currentDonation.from} - {currentDonation.total_amount} STARS
          </span>
        </div>
        {#if currentDonation.message}
          <div class="donation-message">
            {currentDonation.message}
          </div>
        {/if}
      {/if}
    </div>
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

  .donation-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .image-container img {
    max-width: 90vw;
    max-height: 80vh;
    object-fit: contain;
  }

  .donation-main {
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .donation-name {
    font-family: "Roboto Condensed", Tahoma, Arial, sans-serif;
    font-size: 60px;
    color: #fb8c2b;
    font-weight: bold;
    text-align: center;
    vertical-align: middle;
    text-shadow:
      0px 0px 1px #000,
      0px 0px 2px #000,
      0px 0px 3px #000,
      0px 0px 4px #000,
      0px 0px 5px #000;
    letter-spacing: 0px;
    word-spacing: 0px;
    background-color: rgba(255, 255, 255, 0);
    border-radius: 0px;
    padding: 10px 0;
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 90vw;
  }

  .donation-message {
    font-family: "Roboto Condensed", Tahoma, Arial, sans-serif;
    font-size: 25px;
    color: #fff;
    font-weight: normal;
    font-style: normal;
    text-decoration: none;
    text-transform: none;
    text-shadow:
      0px 0px 1px #000,
      0px 0px 2px #000,
      0px 0px 3px #000,
      0px 0px 4px #000,
      0px 0px 5px #000;
    letter-spacing: 0px;
    word-spacing: 0px;
    text-align: center;
    vertical-align: middle;
    background-color: rgba(223, 255, 0, 0); /* полностью прозрачный фон */
    border-radius: 0px;
    padding: 10px;
    margin-top: 10px;
    display: block;
    word-break: break-word;
    max-width: 90vw;
  }

  .hidden {
    opacity: 0;
    pointer-events: none;
  }
</style>
