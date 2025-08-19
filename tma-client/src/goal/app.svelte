<script lang="ts">
  import { Title } from "telegram-ui";
  import { TextStyle, type FontSettings, type GoalProps } from "./types";

  let goalProps: GoalProps = {
    // 1 elements settings
    title: "Donation Goal Title",
    titlePosition: "top",

    progressPosition: "inside",
    progressType: "cur_stars_w_percent",

    displayLimits: true,
    maxLimit: 220,
    minLimit: 0,

    progress: 102,

    displayBackground: false,

    // 2 progress bar design
    barHeight: "9vw",
    roundingRadius: "4vw",
    barStrokeThickness: ".4vw",
    strokeColor: "rgba(255,0,0,0.91)",

    // Background bar styling
    bgBarColor: {
      // colorType: "solid",
      color: "#575757",
      colorType: "gradient",
      gradient: "linear-gradient(0deg, #fc3232, #68f752)",
      // gradient: "linear-gradient(135deg, #667eea, #764ba2)",
      // gradient: "linear-gradient(90deg, #ff9a9e, #fecfef)",
    },

    // Progress bar styling
    progressBarColor: {
      colorType: "gradient",
      gradient: "linear-gradient(90deg, rgb(245, 117, 7), rgb(255, 215, 0))",
      // gradient:  "linear-gradient(90deg, #56ab2f, #a8e6cf)",
      // gradient:  "linear-gradient(45deg, #ff512f, #f09819)",
      // gradient:  "linear-gradient(135deg, #667eea, #764ba2)",
    },

    // 3 font settings
    titleFontSettings: {
      // 1 font
      fontFamily: "Rubik",
      color: "rgb(237, 47, 47)",
      style: [],
      transformation: "uppercase",
      horizontalAlignment: "center",

      // 2 text
      fontSize: "2.75vw",
      lineHeight: "1u",
      letterSpacing: "0vw",
      wordSpacing: "0vw",

      // 3 shadow
      shadowColor: "rgba(0, 0, 0, 0.24)",
      shadowOffsetX: "0.3vw",
      shadowOffsetY: "0.3vw",
      shadowBlur: "0.5vw",
    },
    progressBarFontSettings: {
      // 1 font
      fontFamily: "Rubik",
      color: "#8cded6",
      style: [TextStyle.BOLD],
      // transformation: "uppercase",
      horizontalAlignment: "center",

      // 2 text
      fontSize: "3.65vw",
      lineHeight: "1u",
      letterSpacing: "0vw",
      wordSpacing: "0vw",

      // 3 shadow
      shadowColor: "rgba(0, 0, 0, 0.24)",
      shadowOffsetX: "0.3vw",
      shadowOffsetY: "0.3vw",
      shadowBlur: "0.5vw",
    },
    limitsFontSettings: {
      // 1 font
      fontFamily: "Rubik",
      color: "#63eb67",
      style: [TextStyle.BOLD],
      // transformation: "uppercase",
      horizontalAlignment: "right",

      // 2 text
      fontSize: "4.65vw",
      lineHeight: "1.2u",
      letterSpacing: "-0.2vw",
      wordSpacing: "0vw",

      // 3 shadow
      // shadowColor: "rgba(0, 0, 0, 0.24)",
      // shadowOffsetX: "0.3vw",
      // shadowOffsetY: "0.3vw",
      // shadowBlur: "0.5vw",
    },
  };

  let percentage = $derived((goalProps.progress / goalProps.maxLimit) * 100);
  let displayedPercentage = $derived(Math.floor(percentage));

  let bgBarColor = $derived.by(() => {
    if (
      goalProps.bgBarColor.colorType === "gradient" &&
      goalProps.bgBarColor.gradient
    ) {
      return `background-image: ${goalProps.bgBarColor.gradient}`;
    }
    return `background-color: ${goalProps.bgBarColor.color}`;
  });

  let progressBarColor = $derived.by(() => {
    if (
      goalProps.progressBarColor.colorType === "gradient" &&
      goalProps.progressBarColor.gradient
    ) {
      return `background-image: ${goalProps.progressBarColor.gradient}`;
    }
    return `background-color: ${goalProps.progressBarColor}`;
  });

  let progressValue = $derived.by(() => {
    if (goalProps.progressType === "cur_stars_w_percent") {
      return `${goalProps.progress} STARS (${displayedPercentage}%)`;
    }
    return `${goalProps.progress} STARS`;
  });
</script>

<!-- wrapper -->
<div
  style="flex: 1 0 auto;
    height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    width: 100%;"
>
  <div
    class={goalProps.displayBackground ? "dark-background" : ""}
    style="
    font-family: Arial, sans-serif; 
    margin: 20px; 

    position: relative;
    margin-left: auto;
    margin-right: auto;
    
    width: 80%;
    max-width: 80vw;
    "
  >
    <!-- header -->
    {#if goalProps.titlePosition === "top"}
      {@render title(goalProps.title, goalProps.titleFontSettings)}
    {/if}
    {#if goalProps.progressPosition === "top"}
      {@render progress_info(progressValue, goalProps.progressBarFontSettings)}
    {/if}

    <!-- goal body -->
    <div
      id="goal-body"
      style="
        height: {goalProps.barHeight}; 
        display: flex; flex-direction: column; justify-content: center; align-items: center; 
        position: relative; 
        {goalProps.titlePosition === 'top' ||
      goalProps.progressPosition == 'top'
        ? 'margin-top: .5vw;'
        : ''}"
    >
      <div
        style="width: 100%; 
        height: {goalProps.barHeight}; 
        {bgBarColor}; 
        border-radius: {goalProps.roundingRadius}; 
        overflow: hidden; 
        position: absolute; top: 0; left: 0;         
        border-width: {goalProps.barStrokeThickness};
        border-color: {goalProps.strokeColor};
        border-style: solid;
        box-sizing: border-box;"
      >
        <!-- inside progress line -->
        <div
          style="width: {percentage}%; height: 100%; {progressBarColor}; transition: right 1s; position: absolute; top: 0; left: 0;"
        ></div>
      </div>
      {#if goalProps.titlePosition === "inside"}
        {@render title(goalProps.title, goalProps.titleFontSettings)}
      {/if}
      {#if goalProps.progressPosition === "inside"}
        {@render progress_info(
          progressValue,
          goalProps.progressBarFontSettings
        )}
      {/if}
    </div>

    <!-- footer -->
    <div
      style="
          width: 100%;
          display: flex;
          justify-content: space-between;
          position: relative;"
    >
      {#if goalProps.displayLimits}
        {@render limit(goalProps.minLimit)}
      {/if}

      <div>
        {#if goalProps.titlePosition === "below"}
          {@render title(goalProps.title, goalProps.titleFontSettings)}
        {/if}
        {#if goalProps.progressPosition === "below"}
          {@render progress_info(
            progressValue,
            goalProps.progressBarFontSettings
          )}
        {/if}
      </div>

      {#if goalProps.displayLimits}
        {@render limit(goalProps.maxLimit)}
      {/if}
    </div>
  </div>
</div>

{#snippet title(title: string, fontSettings: FontSettings)}
  <div
    style="    
      color: {fontSettings.color};
      text-transform: {fontSettings.transformation};
      text-align: {fontSettings.horizontalAlignment};
      font-weight: {fontSettings.style?.includes(TextStyle.BOLD)
      ? 'bold'
      : 'normal'};
      font-style: {fontSettings.style?.includes(TextStyle.ITALIC)
      ? 'italic'
      : 'normal'};
      text-decoration: none;
      stroke-width: 0;
      font-family: {fontSettings.fontFamily};
      font-size: {fontSettings.fontSize};
      line-height: {fontSettings.lineHeight};
      letter-spacing: {fontSettings.letterSpacing};
      word-spacing: {fontSettings.wordSpacing};
      width: 100%;
      position: relative;
      z-index: 10;
      {fontSettings.shadowColor
      ? `text-shadow: ${fontSettings.shadowColor} ${fontSettings.shadowOffsetX} ${fontSettings.shadowOffsetY} ${fontSettings.shadowBlur};`
      : ''}
      {fontSettings.horizontalAlignment === 'left'
      ? 'text-align: left;'
      : fontSettings.horizontalAlignment === 'right'
        ? 'text-align: right;'
        : 'text-align: center;'}
      "
  >
    {title}
  </div>
{/snippet}

{#snippet progress_info(value: string, fontSettings: FontSettings)}
  <div
    style="
      color: {fontSettings.color};
      text-transform: none;
      text-align: center;
      font-weight: {fontSettings.style?.includes(TextStyle.BOLD)
      ? 'bold'
      : 'normal'};
      font-style: {fontSettings.style?.includes(TextStyle.ITALIC)
      ? 'italic'
      : 'normal'};
      text-decoration: none;
      {fontSettings.shadowColor
      ? `text-shadow: ${fontSettings.shadowColor} ${fontSettings.shadowOffsetX} ${fontSettings.shadowOffsetY} ${fontSettings.shadowBlur};`
      : ''}
      stroke-width: 0;
      font-family: {fontSettings.fontFamily};
      font-size: {fontSettings.fontSize};
      line-height: {fontSettings.lineHeight};
      letter-spacing: {fontSettings.letterSpacing};
      word-spacing: {fontSettings.wordSpacing};
      width: 100%;
      position: relative;
      z-index: 10;
      {fontSettings.shadowColor
      ? `text-shadow: ${fontSettings.shadowColor} ${fontSettings.shadowOffsetX} ${fontSettings.shadowOffsetY} ${fontSettings.shadowBlur};`
      : ''}
      {fontSettings.horizontalAlignment === 'left'
      ? 'text-align: left;'
      : fontSettings.horizontalAlignment === 'right'
        ? 'text-align: right;'
        : 'text-align: center;'}
      "
  >
    {value}
  </div>
{/snippet}

{#snippet limit(limit: number)}
  <div
    style="
      color: rgb(99, 235, 103);
      text-transform: none;
      text-align: right;
      font-weight: bold;
      font-style: normal;
      text-decoration: none;
      stroke-width: 0;
      font-family: Rubik;
      font-size: 4.65vw;
      line-height: 1.2;
      letter-spacing: -0.2vw;
      word-spacing: 0vw;"
  >
    {limit}
  </div>
{/snippet}

<style>
  .dark-background {
    background-color: rgba(0, 0, 0, 0.6);
    padding: 1.5vw 2vw;
    border-radius: 1vw;
  }
</style>
