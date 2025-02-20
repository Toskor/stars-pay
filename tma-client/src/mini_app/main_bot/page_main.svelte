<script lang="ts">
  import { onMount } from "svelte";
  import type { Page } from "./types";
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

  let isIOS = true;

  onMount(() => {});
</script>

<List platform="ios" style="background-color: var(--tgui--secondary_bg_color)">
  <Section platform="ios">
    {#snippet header()}
      <div class="header-row">
        <SectionHeader platform="ios">Your bots</SectionHeader>
        <Button mode="plain" size="s" onclick={() => navigateTo("create")}
          >Add</Button
        >
      </div>
    {/snippet}

    {#snippet children()}
      <Cell platform="ios">
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

      <Cell platform="ios">
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

  <Section platform="ios" header="Suspended bots">
    <Cell platform="ios">
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
