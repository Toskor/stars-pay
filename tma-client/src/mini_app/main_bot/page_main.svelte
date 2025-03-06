<script lang="ts">
  import { onMount } from "svelte";
  import { type MainPageProps, type Page, type Bot } from "./types";
  import {
    Button,
    Title,
    Avatar,
    Cell,
    List,
    Section,
    SectionHeader,
    Divider,
    Skeleton,
    Image,
    QuestionMarkIcon,
    Input,
    AddCircleIcon,
  } from "telegram-ui";
  import { get } from "svelte/store";
  import {
    botsStore,
    refreshBotsData,
    getAvatarAsObjectUrl,
    type BotsStoreType,
  } from "./store";

  let { navigateTo }: { navigateTo: (page: Page, bot?: Bot) => void } =
    $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let data = $state<MainPageProps | null>(null);
  let loadTime = $state<number | null>(null);
  let isRefreshing = $state(false);

  let suspendedBots = $state<Bot[]>([]);

  async function handleRefresh() {
    isRefreshing = true;
    try {
      await refreshBotsData(app.initData);

      const storeValue = get(botsStore) as BotsStoreType;
      error = storeValue.error;
      data = storeValue.data;
      loadTime = storeValue.loadTime;

      if (data?.bots) {
        suspendedBots = data.bots.filter((bot) => bot.suspended);
      }
    } finally {
      isRefreshing = false;
    }
  }

  onMount(() => {
    const storeValue = get(botsStore) as BotsStoreType;

    isLoading = storeValue.isLoading;
    error = storeValue.error;
    data = storeValue.data;
    loadTime = storeValue.loadTime;

    if (data?.bots) {
      suspendedBots = data.bots.filter((bot) => bot.suspended);
    }

    const unsubscribe = botsStore.subscribe((store: BotsStoreType) => {
      isLoading = store.isLoading;
      error = store.error;
      data = store.data;
      loadTime = store.loadTime;

      if (data?.bots) {
        suspendedBots = data.bots.filter((bot) => bot.suspended);
      }
    });

    return unsubscribe;
  });
</script>

{#snippet placeholder()}
  <div style="height: 75px; width: 100%;"></div>
{/snippet}

{#if isLoading}
  <Skeleton style="margin-bottom: 12px;">
    {@render placeholder()}
  </Skeleton>

  <Skeleton style="margin-bottom: 12px;">
    {@render placeholder()}
  </Skeleton>

  <Skeleton>
    {@render placeholder()}
  </Skeleton>
{:else if error}
  <div class="error-container">
    <div class="error-text">Error: {error}</div>
    <Button mode="filled" size="m" onclick={() => location.reload()}>
      Retry
    </Button>
  </div>
{:else}
  <List>
    <Section>
      {#snippet header()}
        <div class="header-row">
          <SectionHeader>Your bots</SectionHeader>
          <Button mode="plain" size="s" onclick={() => navigateTo("create")}
            >Add</Button
          >
        </div>
      {/snippet}

      {#snippet children()}
        {#if data?.bots.length == 0}
          <div class="layout">
            <Image
              src="https://avatars.mds.yandex.net/i?id=0b56680182693c18b90b7e5047abbe27db7bd586-9198264-images-thumbs&n=13"
              size={128}
            ></Image>

            <div class="layout-horizontal">
              <Title weight={2} level={2}>You have no bot yet</Title>
            </div>

            <Button
              mode="filled"
              size="l"
              stretched={true}
              onclick={() => navigateTo("create")}
            >
              Add one
              {#snippet before()}
                <AddCircleIcon isFill={true} />
              {/snippet}
            </Button>
          </div>
        {:else}
          {#each data?.bots || [] as bot, index}
            <Cell>
              {#snippet before()}
                <Avatar size={48} src={bot.avatar} acronym={bot.name[0]} />
              {/snippet}

              {#snippet after()}
                <Button
                  mode="bezeled"
                  size="s"
                  onclick={() => navigateTo("edit", bot)}>Edit</Button
                >
              {/snippet}

              {#snippet children()}
                {bot.name}
              {/snippet}

              {#snippet subtitle()}
                {bot.userRole}
              {/snippet}
            </Cell>

            {#if index < (data?.bots?.length || 0) - 1}
              <Divider />
            {/if}
          {/each}
        {/if}
      {/snippet}
    </Section>

    {#if suspendedBots.length > 0}
      <Section>
        {#snippet header()}
          <SectionHeader>Suspended bots</SectionHeader>
        {/snippet}

        {#snippet children()}
          {#each suspendedBots as bot, index}
            <Cell>
              {#snippet before()}
                <Avatar size={48} src={bot.avatar} />
              {/snippet}

              {#snippet children()}
                {bot.name}
              {/snippet}

              {#snippet subtitle()}
                Need to pay {bot.debt || 100} stars
              {/snippet}

              {#snippet description()}
                <!-- todo button action -->
                <Button mode="filled" size="s">Pay</Button>
              {/snippet}
            </Cell>

            {#if index < suspendedBots.length - 1}
              <Divider />
            {/if}
          {/each}
        {/snippet}
      </Section>
    {/if}
  </List>
{/if}

<style>
  .error-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 24px;
  }

  .error-text {
    color: var(--tg-theme-text-color);
    text-align: center;
  }
</style>
