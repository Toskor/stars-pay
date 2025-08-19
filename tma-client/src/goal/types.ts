export interface GoalProps {
  // 1 elements settings
  // title
  title: string;
  titlePosition: "top" | "inside" | "below" | "invisible";

  // progress
  progressPosition: "top" | "inside" | "below" | "invisible";
  progressType:
    | "percent"
    | "cur_stars"
    | "cur_stars_w_percent"
    | "cur_stars/target_stars"
    | "cur_stars/target_stars_w_percent";

  // goal limits
  displayLimits: boolean;
  maxLimit: number;
  minLimit: number;
  progress: number;

  //background
  displayBackground: boolean;

  // 2 progress bar design
  barHeight: string;
  roundingRadius: string;

  barStrokeThickness: string;
  strokeColor: string;

  // Background bar styling
  bgBarColor: ColorSettings;

  // Progress bar styling
  progressBarColor: ColorSettings;

  // 3 font settings
  titleFontSettings: FontSettings;
  progressBarFontSettings: FontSettings;
  limitsFontSettings: {};
}

export interface ColorSettings {
  colorType: "solid" | "gradient";
  color?: string;
  gradient?: string;
}

export interface FontSettings {
  // 1 font
  fontFamily: string;
  color: string;
  style?: TextStyle[];
  transformation?: "uppercase" | "lowercase";
  horizontalAlignment: "left" | "center" | "right";

  // 2 text
  fontSize: string;
  lineHeight: string;
  letterSpacing: string;
  wordSpacing: string;

  // 3 shadow
  // It is difficult to make out a dark shadow on a dark background.
  shadowColor?: string;
  shadowOffsetX?: string;
  shadowOffsetY?: string;
  shadowBlur?: string;
}

export enum TextStyle {
  BOLD = "bold",
  ITALIC = "italic",
}