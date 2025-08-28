<script lang="ts">
  import { onMount } from "svelte";
  import { TextStyle, type FontSettings, type GoalProps } from "./types";

  onMount(() => {
    // document.body.style.height = "100vh";
    // document.body.style.width = "100vw";
    // document.body.style.overflow = "hidden";
  });

  let { goalProps }: { goalProps: GoalProps } = $props();

  let percentage = $derived((goalProps.progress / goalProps.maxLimit) * 100);
  let displayedPercentage = $derived(Math.floor(percentage));

  let bgBarColor = $derived.by(() => {
    if (
      goalProps.bgBarColor.colorType === "gradient"
    ) {
      return `background-image: ${goalProps.bgBarColor.color}`;
    }
    return `background-color: ${goalProps.bgBarColor.color}`;
  });

  let progressBarColor = $derived.by(() => {
    if (
      goalProps.progressBarColor.colorType === "gradient"
    ) {
      return `background-image: ${goalProps.progressBarColor.color}`;
    }
    return `background-color: ${goalProps.progressBarColor.color}`;
  });

  let verticalProgress = $derived(100 - percentage);

  let progressValue = $derived.by(() => {
    switch (goalProps.progressType) {
      case "percent":
        return `${displayedPercentage}%`;
      case "cur_stars":
        return `${goalProps.progress} STARS`;
      case "cur_stars_w_percent":
        return `${goalProps.progress} STARS (${displayedPercentage}%)`;
      case "cur_stars/target_stars":
        return `${goalProps.progress} / ${goalProps.maxLimit} STARS`;
      case "cur_stars/target_stars_w_percent":
        return `${goalProps.progress} / ${goalProps.maxLimit} STARS (${displayedPercentage}%)`;
      default:
        return `${goalProps.progress} STARS`;
    }
  });
</script>

<!-- wrapper -->
<div
  style="flex: 1 0 auto;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    width: 100%;"
>
  <div
    class="{goalProps.displayBackground
      ? 'dark-background'
      : ''} {goalProps.isVertical ? 'vertical' : ''}"
    style="
    font-family: Arial, sans-serif; 
    margin: 20px; 

    position: relative;
    margin-left: auto;
    margin-right: auto;
    
    {goalProps.isVertical
      ? 'height: 80%; max-height: 80vh; width: auto;'
      : 'width: 80%; max-width: 80vw; height: auto;'}
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
        {goalProps.isVertical
        ? `width: 25vw; height: ${goalProps.barHeight}vw; flex: 1;`
        : `height: ${goalProps.barHeight}vw; width: 100%;`}
        display: flex; 
        {goalProps.isVertical
        ? 'flex-direction: column;'
        : 'flex-direction: column;'} 
        justify-content: center; align-items: center; 
        position: relative; 
        {goalProps.titlePosition === 'top' ||
      goalProps.progressPosition == 'top'
        ? 'margin-top: .5vw;'
        : ''}"
    >
      <div
        style="{goalProps.isVertical
          ? `height: ${goalProps.barHeight}vw; width: 25vw;`
          : `width: 100%; height: ${goalProps.barHeight}vw;`}
        {bgBarColor}; 
        border-radius: {goalProps.roundingRadius}vw; 
        overflow: hidden; 
        position: absolute; top: 0; left: 0;         
        border-width: {goalProps.barStrokeThickness}vw;
        border-color: {goalProps.strokeColor};
        border-style: solid;
        box-sizing: border-box;"
      >
        <!-- inside progress line -->
        <div
          style="{goalProps.isVertical
            ? `width: 100%; height: ${percentage}%; top: ${verticalProgress}%; left: 0; transition: top 1s;`
            : `width: ${percentage}%; height: 100%; top: 0; left: 0; transition: width 1s;`} 
          {progressBarColor}; 
          position: absolute;"
        ></div>
      </div>

      {#if goalProps.titlePosition === "inside" || goalProps.progressPosition === "inside"}
        <div
          class="inside-text-container {goalProps.isVertical
            ? 'vertical-inside'
            : 'horizontal-inside'}"
        >
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
      {/if}
    </div>

    <!-- footer -->
    <div
      style="
          {goalProps.isVertical
        ? 'height: 100%; flex-direction: column; align-items: center;'
        : 'width: 100%; flex-direction: row;'}
          display: flex;
          justify-content: space-between;
          position: relative;"
    >
      {#if goalProps.displayLimits}
        {@render limit(
          goalProps.isVertical ? goalProps.maxLimit : goalProps.minLimit,
          goalProps.limitsFontSettings,
          true,
          goalProps.isVertical
        )}
      {/if}

      {#if goalProps.titlePosition === "below"}
        {@render title(goalProps.title, goalProps.titleFontSettings)}
      {/if}
      {#if goalProps.progressPosition === "below"}
        {@render progress_info(
          progressValue,
          goalProps.progressBarFontSettings
        )}
      {/if}

      {#if goalProps.displayLimits}
        {@render limit(
          goalProps.isVertical ? goalProps.minLimit : goalProps.maxLimit,
          goalProps.limitsFontSettings,
          false,
          goalProps.isVertical
        )}
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
      font-size: {fontSettings.fontSize}vw;
      line-height: {fontSettings.lineHeight}u;
      letter-spacing: {fontSettings.letterSpacing}vw;
      word-spacing: {fontSettings.wordSpacing}vw;
      width: 100%;
      position: relative;
      z-index: 10;
      {fontSettings.shadowColor
      ? `text-shadow: ${fontSettings.shadowColor} ${fontSettings.shadowOffsetX}vw ${fontSettings.shadowOffsetY}vw ${fontSettings.shadowBlur}vw;`
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
      ? `text-shadow: ${fontSettings.shadowColor} ${fontSettings.shadowOffsetX}vw ${fontSettings.shadowOffsetY}vw ${fontSettings.shadowBlur}vw;`
      : ''}
      stroke-width: 0;
      font-family: {fontSettings.fontFamily};
      font-size: {fontSettings.fontSize}vw;
      line-height: {fontSettings.lineHeight}u;
      letter-spacing: {fontSettings.letterSpacing}vw;
      word-spacing: {fontSettings.wordSpacing}vw;
      width: 100%;
      position: relative;
      z-index: 10;
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

{#snippet limit(
  limit: number,
  fontSettings: FontSettings,
  isMin: boolean,
  isVertical: boolean
)}
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
      ? `text-shadow: ${fontSettings.shadowColor} ${fontSettings.shadowOffsetX}vw ${fontSettings.shadowOffsetY}vw ${fontSettings.shadowBlur}vw;`
      : ''}
      stroke-width: 0;
      font-family: {fontSettings.fontFamily};
      font-size: {fontSettings.fontSize}vw;
      line-height: {fontSettings.lineHeight}u;
      letter-spacing: {fontSettings.letterSpacing}vw;
      word-spacing: {fontSettings.wordSpacing}vw;
      width: 100%;
      position: relative;
      z-index: 10;
      {isVertical
      ? 'text-align: center;'
      : isMin
        ? 'text-align: left;'
        : 'text-align: right;'}
      "
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

  .vertical {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .vertical #goal-body {
    flex: 1;
    min-height: 0;
  }

  .inside-text-container {
    position: absolute;
    z-index: 20;
    display: flex;
    justify-content: center;
    align-items: center;
    pointer-events: none;
  }

  .vertical-inside {
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    flex-direction: column;
    gap: 0.5rem;
  }

  .horizontal-inside {
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    flex-direction: column;
    gap: 0.5rem;
  }
</style>
