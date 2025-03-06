<script lang="ts">
  import { onMount } from "svelte";

  import {
    Button,
    Text,
    Title,
    Accordion,
    AccordionSummary,
    AccordionContent,
    Avatar,
    Cell,
    List,
    Section,
    SectionHeader,
    Divider,
    ForwardIcon,
    EditIcon,
    DeleteIcon,
  } from "telegram-ui";
  import type { Page, Bot } from "./types";
  let {
    navigateTo,
    bot,
  }: { navigateTo: (page: Page, bot?: Bot) => void; bot: Bot | null } =
    $props();

  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");

  onMount(() => {
    if (!bot) {
      navigateTo("main");
    }
  });
</script>

<List>
  <div class="header--center-container">
    <Avatar size={128} src={bot?.avatar || ""} acronym={bot?.name?.[0] || ""} />
  </div>

  <Section header="Owner">
    <Cell>
      {#snippet before()}
        <Avatar
          size={48}
          src={bot?.owner?.avatarUrl || ""}
          acronym={bot?.owner?.name?.[0] || ""}
        />
      {/snippet}

      {#snippet children()}
        {bot?.owner?.name || ""}
      {/snippet}

      {#snippet subtitle()}
        {bot?.owner?.username || ""}
      {/snippet}
    </Cell>
  </Section>

  <Section>
    {#snippet header()}
      <div class="header-row">
        <SectionHeader>Bot admins</SectionHeader>
        <Button mode="plain" size="s">Add</Button>
      </div>
    {/snippet}

    {#if bot?.admins && bot.admins.length > 0}
      {#each bot.admins as admin, index}
        <Cell>
          {#snippet before()}
            <Avatar
              size={48}
              src={admin.avatarUrl || ""}
              acronym={admin.name?.[0] || ""}
            />
          {/snippet}

          {#snippet after()}
            <Button mode="destructive" size="s">Remove</Button>
          {/snippet}

          {#snippet children()}
            {admin.name || ""}
          {/snippet}

          {#snippet subtitle()}
            {admin.username || ""}
          {/snippet}
        </Cell>

        {#if index < bot.admins.length - 1}
          <Divider />
        {/if}
      {/each}
    {:else}
      <Cell>
        <div class="no-admins">No admins added yet</div>
      </Cell>
    {/if}
  </Section>

  <Section>
    <Accordion>
      <AccordionSummary>
        <Text weight={3}>Danger Zone</Text>
      </AccordionSummary>

      <AccordionContent style="color: var(--tgui--destructive_text_color);">
        <Cell
          onclick={() => {
            console.log("change owner");
          }}
        >
          {#snippet before()}
            <ForwardIcon />
          {/snippet}
          Change Owner
        </Cell>

        <Divider />

        <Cell
          onclick={() => {
            console.log("change bot token");
          }}
        >
          {#snippet before()}
            <EditIcon />
          {/snippet}
          Change Bot Token
        </Cell>

        <Divider />

        <Cell
          onclick={() => {
            console.log("remove bot");
          }}
        >
          {#snippet before()}
            <DeleteIcon />
          {/snippet}
          Remove Bot
        </Cell>
      </AccordionContent>
    </Accordion>
  </Section>
</List>

<style>
</style>
