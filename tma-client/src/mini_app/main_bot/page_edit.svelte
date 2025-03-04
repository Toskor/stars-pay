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
  import type { Page } from "./types";
  let { navigateTo }: { navigateTo: (page: Page) => void } = $props();

  let platform = $state<"ios" | "base">("ios");
  let isIOS = $derived(platform === "ios");

  onMount(() => {});
</script>

<List>
  <div class="header--center-container">
    <Avatar
      size={128}
      src="https://i.pinimg.com/originals/d0/cf/a8/d0cfa8b3f2b9aa687e99cdd88bb82f10.jpg"
    />
  </div>

  <Section header="Owner">
    <Cell>
      {#snippet before()}
        <Avatar
          size={48}
          src="https://avatars.githubusercontent.com/u/84640980?v=4"
        />
      {/snippet}

      {#snippet children()}
        Yury Korolev
      {/snippet}

      {#snippet subtitle()}
        88005553535
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

    <Cell>
      {#snippet before()}
        <Avatar
          size={48}
          src="https://i.pinimg.com/originals/d0/cf/a8/d0cfa8b3f2b9aa687e99cdd88bb82f10.jpg"
        />
      {/snippet}

      {#snippet after()}
        <Button mode="destructive" size="s">Remove</Button>
      {/snippet}

      {#snippet children()}
        Username 1
      {/snippet}

      {#snippet subtitle()}
        2364409283
      {/snippet}
    </Cell>

    <Divider />

    <Cell>
      {#snippet before()}
        <Avatar
          size={48}
          src="https://i1.sndcdn.com/artworks-000041319143-zgx70e-t500x500.jpg"
        />
      {/snippet}

      {#snippet after()}
        <Button mode="destructive" size="s">Remove</Button>
      {/snippet}

      {#snippet children()}
        Username 2
      {/snippet}

      {#snippet subtitle()}
        2364409283
      {/snippet}
    </Cell>
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
