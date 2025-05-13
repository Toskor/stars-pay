<script lang="ts">
  import "telegram-ui/styles";
  import { Title, Text, Section, platform } from "telegram-ui";
  import { onMount } from "svelte";

  //@ts-ignore
  //   let app = window.Telegram.WebApp;

  onMount(() => {
    // if (app) {
    //   app.expand();
    // }

    let isIOS = platform() === "ios";

    document.body.classList.add("wrapper");
    document.body.classList.add("wrapper--horizontal-limit");
    if (isIOS) {
      document.body.classList.add("wrapper-ios");
    }

    return () => {
      document.body.classList.remove("wrapper");
      document.body.classList.remove("wrapper--horizontal-limit");
      if (isIOS) {
        document.body.classList.remove("wrapper-ios");
      }
    };
  });
</script>

<div class="layout center-container">
  <Title weight={2} level={2}>⚠️ Service Temporarily Unavailable</Title>
  <Text>
    Unfortunately, the service is temporarily not working. Please notify the
    streamer about this.
  </Text>
</div>

<style>
  :global(body) {
    user-select: none;
  }

  .center-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 95vh;
    max-width: 90vw;
    padding: 16px;
    text-align: center;
  }
</style>
