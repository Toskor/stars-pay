<script lang="ts">
  import { onMount } from "svelte";
  //@ts-ignore
  import {
    Slider,
    Accordion,
    List,
    AccordionSummary,
    Section,
    SectionHeader,
    Input,
    AccordionContent,
    CopyIcon,
    ChevronDownIcon,
    Button,
  } from "telegram-ui";
  import { TextStyle, type GoalProps } from "../../goal/types";
  import Goal from "../../goal/goal.svelte";

  let goalProps: GoalProps = $state({
    url: "https://tg-stars.s3-website.nl-ams.scw.cloud/goal/some-uuid",
    // 0 general settings
    title: "Donation Goal Title",
    progress: 102,
    maxLimit: 400,

    // 1 elements settings
    titlePosition: "top",

    progressPosition: "inside",
    progressType: "cur_stars_w_percent",

    displayLimits: true,
    minLimit: 0,

    displayBackground: false,
    isVertical: false,

    // 2 progress bar design
    barHeight: 15,
    roundingRadius: 4,
    barStrokeThickness: 0.4,
    strokeColor: "rgba(255,0,0,0.91)",

    // Background bar styling
    bgBarColor: {
      colorType: "solid",
      color: "#575757", // gray
      //   colorType: "gradient",
      //   gradient: "linear-gradient(0deg, #fc3232, #68f752)",
      // gradient: "linear-gradient(135deg, #667eea, #764ba2)",
      // gradient: "linear-gradient(90deg, #ff9a9e, #fecfef)",
    },

    // Progress bar styling
    progressBarColor: {
      colorType: "solid",
      color: "rgb(255, 215, 0)",
      //   colorType: "gradient",
      //   gradient: "linear-gradient(90deg, rgb(245, 117, 7), rgb(255, 215, 0))",
      // gradient:  "linear-gradient(90deg, #56ab2f, #a8e6cf)",
      // gradient:  "linear-gradient(45deg, #ff512f, #f09819)",
      // gradient:  "linear-gradient(135deg, #667eea, #764ba2)",
    },

    // 3 font settings
    titleFontSettings: {
      // 1 font
      fontFamily: "Rubik",
      color: "rgb(237, 67, 47)",
      style: [],
      transformation: "uppercase",
      horizontalAlignment: "center",

      // 2 text
      fontSize: 2.75,
      lineHeight: 1,
      letterSpacing: 0,
      wordSpacing: 0,

      // 3 shadow
      shadowColor: "rgba(0, 0, 0, 0.24)",
      shadowOffsetX: 0.3,
      shadowOffsetY: 0.3,
      shadowBlur: 0.5,
    },
    progressBarFontSettings: {
      // 1 font
      fontFamily: "Rubik",
      color: "#8cded6",
      style: [TextStyle.BOLD],
      // transformation: "uppercase",
      horizontalAlignment: "center",

      // 2 text
      fontSize: 3.65,
      lineHeight: 1,
      letterSpacing: 0,
      wordSpacing: 0,

      // 3 shadow
      shadowColor: "rgba(0, 0, 0, 0.24)",
      shadowOffsetX: 0.3,
      shadowOffsetY: 0.3,
      shadowBlur: 0.5,
    },
    limitsFontSettings: {
      // 1 font
      fontFamily: "Rubik",
      color: "#63eb67",
      style: [TextStyle.BOLD],
      // transformation: "uppercase",
      horizontalAlignment: "right",

      // 2 text
      fontSize: 4.65,
      lineHeight: 1.2,
      letterSpacing: -0.2,
      wordSpacing: 0,

      // 3 shadow
      // shadowColor: "rgba(0, 0, 0, 0.24)",
      // shadowOffsetX: 0.3,
      // shadowOffsetY: 0.3,
      // shadowBlur: 0.5,
    },
  });

  let isCopied = $state(false);
  function copyUrlToClipboard() {
    if (navigator && navigator.clipboard) {
      navigator.clipboard
        .writeText(goalProps.url)
        .then(() => {
          isCopied = true;
          setTimeout(() => {
            isCopied = false;
          }, 2000);
        })
        .catch((err) => {
          console.error("Error copying URL:", err);
        });
    }
  }

  onMount(() => {
    console.log("goalProps", goalProps);
    document.body.classList.add("wrapper");
  });
</script>

<div class="main-preview" style="background-color: ;">
  <Goal {goalProps} />
</div>

<Section>
  <SectionHeader>Goal editor</SectionHeader>
  <List>
    <Button mode="filled" size="s" onclick={copyUrlToClipboard}>
      {#snippet after()}
        {#if isCopied}
          <div class="icon fade-in">
            <ChevronDownIcon />
          </div>
        {:else}
          <div class="icon fade-in">
            <CopyIcon isFill={true} />
          </div>
        {/if}
      {/snippet}
      Copy link
    </Button>

    <!-- <Input bind:value={url} header="URL">
      {#snippet after()}
        <Button mode="filled" size="s" onclick={copyUrlToClipboard}>
          {#if isCopied}
            <div class="icon fade-in">
              <ChevronDownIcon />
            </div>
          {:else}
            <div class="icon fade-in">
              <CopyIcon isFill={true} />
            </div>
          {/if}
        </Button>
      {/snippet}
    </Input> -->
    <!-- 0 general settings -->
    <Accordion>
      <AccordionSummary>General settings</AccordionSummary>
      <AccordionContent>
        <Input
          bind:value={goalProps.title}
          header="Title"
          placeholder="Enter goal title"
        />

        <Input
          type="number"
          bind:value={goalProps.progress}
          header="Current progress"
          placeholder="Current progress value"
        />
        <!-- style="padding-bottom: 0;" -->
        <Input
          type="number"
          bind:value={goalProps.maxLimit}
          header="Maximum limit"
          placeholder="Maximum goal value"
        />
      </AccordionContent>
    </Accordion>
    <!-- 1 elements settings -->
    <Accordion>
      <AccordionSummary>Elements settings</AccordionSummary>
      <AccordionContent>
        <List style="display: flex; flex-direction: column; gap: 10px;">
          <!-- Title position -->
          <select bind:value={goalProps.titlePosition}>
            <option value="top">Top</option>
            <option value="inside">Inside</option>
            <option value="below">Below</option>
            <option value="invisible">Invisible</option>
          </select>
          <!-- progress position -->
          <select bind:value={goalProps.progressPosition}>
            <option value="top">Top</option>
            <option value="inside">Inside</option>
            <option value="below">Below</option>
            <option value="invisible">Invisible</option>
          </select>
          <!-- progressType -->
          <select bind:value={goalProps.progressType}>
            <option value="percent">Percent</option>
            <option value="cur_stars">Current stars</option>
            <option value="cur_stars_w_percent"
              >Current stars with percent</option
            >
            <option value="cur_stars/target_stars"
              >Current stars / target stars</option
            >
            <option value="cur_stars/target_stars_w_percent"
              >Current stars / target stars with percent</option
            >
          </select>
          <!-- display limits -->
          <select bind:value={goalProps.displayLimits}>
            <option value="true">True</option>
            <option value="false">False</option>
          </select>
          <!-- display background -->
          <select bind:value={goalProps.displayBackground}>
            <option value="true">True</option>
            <option value="false">False</option>
          </select>
        </List>
      </AccordionContent>
    </Accordion>
    <!-- 2 progress bar design -->
    <Accordion>
      <AccordionSummary>Progress bar design</AccordionSummary>
      <AccordionContent>
        <List style="display: flex; flex-direction: column; gap: 10px;">
          <!-- bar height -->
          <div>Bar height</div>
          <Slider bind:value={goalProps.barHeight} min={1} max={30} />
          <!-- rounding radius -->
          <div>Rounding radius</div>
          <Slider bind:value={goalProps.roundingRadius} min={0} max={10} />
          <!-- bar stroke thickness -->
          <div>Stroke thickness</div>
          <Slider bind:value={goalProps.barStrokeThickness} min={0} max={1} />
          <!-- stroke color -->
          <div>Stroke color</div>
          <input
            type="color"
            bind:value={goalProps.strokeColor}
            defaultValue={goalProps.strokeColor}
          />
          <!-- Background bar color: -->
          <div>Background bar color</div>
          <input
            type="color"
            bind:value={goalProps.bgBarColor.color}
            defaultValue={goalProps.bgBarColor.color}
          />
          <!-- progress bar color -->
          <div>Progress bar color</div>
          <input
            type="color"
            bind:value={goalProps.progressBarColor.color}
            defaultValue={goalProps.progressBarColor.color}
          />
        </List>
      </AccordionContent>
    </Accordion>
    <!-- 3 font settings -->
    <Accordion>
      <AccordionSummary>Font settings</AccordionSummary>
      <AccordionContent></AccordionContent>
    </Accordion>
  </List>
</Section>

<style>
  .main-preview {
    position: relative;
    top: 0;
    left: 0;
    width: 100%;
    height: 25vh;
    z-index: 1000;
  }

  .icon {
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.8) rotate(-10deg);
    }
    to {
      opacity: 1;
      transform: scale(1) rotate(0deg);
    }
  }

  /* Additional smooth transition for icon changes */
  .icon-container :global(.icon) {
    transition:
      opacity 0.2s ease-in-out,
      transform 0.2s ease-in-out;
  }
</style>
