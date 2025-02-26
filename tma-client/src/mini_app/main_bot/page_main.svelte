<script lang="ts">
  import { onMount } from "svelte";
  import { type MainPageProps, type Page, type Bot, testBots } from "./types";
  import { getControlledBots } from "./queries";
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
  } from "telegram-ui";

  let { navigateTo }: { navigateTo: (page: Page) => void } = $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let data = $state<MainPageProps | null>(null);

  let suspendedBots = $state<Bot[]>([]);

  onMount(async () => {
    try {
      const result = await getControlledBots(app.initData);
      if (result.success) {
        data = result.data;
        //todo remove (for testing)
        // data = {
        //   bots: testBots,
        // };
        console.log(data);

        if (data.bots) {
          suspendedBots = data.bots.filter((bot) => bot.suspended);
        }
      } else {
        error = result.error;
      }
    } finally {
      isLoading = false;
    }
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
  <!-- <span>{JSON.stringify(data)}</span> -->
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
        {#each data?.bots || [] as bot, index}
          <Cell>
            {#snippet before()}
              <Avatar size={48} src={bot.avatar} acronym={bot.name[0]} />
            {/snippet}

            {#snippet after()}
              <Button mode="bezeled" size="s" onclick={() => navigateTo("edit")}
                >Edit</Button
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
