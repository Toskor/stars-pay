<script lang="ts">
  import type { Page } from "./types";
  import { onMount } from "svelte";
  import {
    Button,
    Divider,
    Title,
    Text,
    Image,
    Input,
    AddCircleIcon,
    QuestionMarkIcon,
    Section,
    SectionFooter,
  } from "telegram-ui";
  let { navigateTo }: { navigateTo: (page: Page) => void } = $props();

  onMount(() => {});

  let value = $state("");
  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");
</script>

<div class="layout">
  <Image
    src="https://avatars.mds.yandex.net/i?id=0b56680182693c18b90b7e5047abbe27db7bd586-9198264-images-thumbs&n=13"
    size={128}
  ></Image>

  <div class="layout-horizontal">
    <Title weight={1} level={3}>Register new bot</Title>
    <QuestionMarkIcon style="color: var(--tgui--link_color);" />
  </div>

  <Input
    platform="ios"
    placeholder="Your bot token"
    header="Input"
    bind:value
    stretched={true}
  />

  <SectionFooter {platform} centered={false}
    >Create bot with @botfather and paste here bot token
    <br />
    You will be owner of this bot by default</SectionFooter
  >

  <Button
    mode="filled"
    size="l"
    stretched={true}
    onclick={() => console.log("clicked")}
  >
    Register Bot

    {#snippet before()}
      <AddCircleIcon isFill={true} />
    {/snippet}
  </Button>
</div>
