<script lang="ts">
  import { onMount } from "svelte";
  import type { MainPageProps, Page } from "./types";
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
  } from "telegram-ui";

  let { navigateTo }: { navigateTo: (page: Page) => void } = $props();

  //@ts-ignore
  let app = window.Telegram.WebApp;

  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let data = $state<MainPageProps | null>(null);

  onMount(async () => {
    try {
      const result = await getControlledBots(app.initData);
      if (result.success) {
        data = result.data;
      } else {
        error = result.error;
      }
    } finally {
      isLoading = false;
    }
  });
</script>

{#if isLoading}
  <div class="loading-container">
    <div class="loading-text">Loading bots...</div>
  </div>
{:else if error}
  <div class="error-container">
    <div class="error-text">Error: {error}</div>
    <Button mode="filled" size="m" onclick={() => location.reload()}>
      Retry
    </Button>
  </div>
{:else}
  <span>{JSON.stringify(data)}</span>
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
        <Cell>
          {#snippet before()}
            <Avatar
              size={48}
              src="https://avatars.githubusercontent.com/u/84640980?v=4"
            />
          {/snippet}

          {#snippet after()}
            <Button mode="bezeled" size="s" onclick={() => navigateTo("edit")}
              >Edit</Button
            >
          {/snippet}

          {#snippet children()}
            YomlDevBot
          {/snippet}

          {#snippet subtitle()}
            Owner
          {/snippet}
        </Cell>

        <Divider />

        <Cell>
          {#snippet before()}
            <Avatar
              size={48}
              src="https://steamuserimages-a.akamaihd.net/ugc/2100422066956953334/BCFFD0DB0C56F71CD288304540E39FC2FADFD155/?imw=512&imh=341&ima=fit&impolicy=Letterbox&imcolor=%23000000&letterbox=true"
            />
          {/snippet}

          {#snippet after()}
            <Button mode="bezeled" size="s" onclick={() => navigateTo("edit")}
              >Edit</Button
            >
          {/snippet}

          {#snippet children()}
            StarsBot
          {/snippet}

          {#snippet subtitle()}
            Admin
          {/snippet}
        </Cell>
      {/snippet}
    </Section>

    <Section header="Suspended bots">
      <Cell>
        {#snippet before()}
          <Avatar
            size={48}
            src="https://i.pinimg.com/originals/d0/cf/a8/d0cfa8b3f2b9aa687e99cdd88bb82f10.jpg"
          />
        {/snippet}

        {#snippet children()}
          AliciaStarsBot
        {/snippet}

        {#snippet subtitle()}
          Need to pay 100$
        {/snippet}

        {#snippet description()}
          <!-- todo button action -->
          <Button mode="filled" size="s">Pay</Button>
        {/snippet}
      </Cell>
    </Section>
  </List>
{/if}
