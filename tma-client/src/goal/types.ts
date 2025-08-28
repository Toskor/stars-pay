export interface GoalProps {
  url: string;
  // 0 general settings
  title: string;
  maxLimit: number;
  progress: number;

  // 1 elements settings
  titlePosition: "top" | "inside" | "below" | "invisible";

  progressPosition: "top" | "inside" | "below" | "invisible";
  progressType:
    | "percent"
    | "cur_stars"
    | "cur_stars_w_percent"
    | "cur_stars/target_stars"
    | "cur_stars/target_stars_w_percent";

  displayLimits: boolean;
  minLimit: number;

  displayBackground: boolean;
  isVertical: boolean;

  // 2 progress bar design
  barHeight: number;
  roundingRadius: number;

  barStrokeThickness: number;
  strokeColor: string;

  bgBarColor: ColorSettings;

  progressBarColor: ColorSettings;

  // 3 font settings
  titleFontSettings: FontSettings;
  progressBarFontSettings: FontSettings;
  limitsFontSettings: FontSettings;
}

export interface ColorSettings {
  colorType: "solid" | "gradient";
  color: string;
}

export interface FontSettings {
  // 1 font
  fontFamily: string;
  color: string;
  style?: TextStyle[];
  transformation?: "uppercase" | "lowercase";
  horizontalAlignment: "left" | "center" | "right";

  // 2 text
  fontSize: number;
  lineHeight: number;
  letterSpacing: number;
  wordSpacing: number;

  // 3 shadow
  // It is difficult to make out a dark shadow on a dark background.
  shadowColor?: string;
  shadowOffsetX?: number;
  shadowOffsetY?: number;
  shadowBlur?: number;
}

export enum TextStyle {
  BOLD = "bold",
  ITALIC = "italic",
}

export const defaultProps: GoalProps = {
  url: 'https://tg-stars.s3-website.nl-ams.scw.cloud/goal/some-uuid',
  // 0 general settings
  title: "Donation Goal Title",
  maxLimit: 444,
  progress: 106,

  // 1 elements settings
  titlePosition: "inside",

  progressPosition: "inside",
  progressType: "cur_stars_w_percent",

  displayLimits: false,
  minLimit: 0,

  displayBackground: false,

  isVertical: false,

  // 2 progress bar design
  barHeight: 29,
  roundingRadius: 4,
  barStrokeThickness: 0.4,
  strokeColor: "rgba(255,0,0,0.91)",

  // Background bar styling
  bgBarColor: {
    colorType: "solid",
    color: "#424242",
    },

  // Progress bar styling
  progressBarColor: {
    colorType: "gradient",
    color: "linear-gradient(0deg, #f57507,rgb(255, 248, 235))",
    },

  // 3 font settings
  titleFontSettings: {
    // 1 font
    fontFamily: "Rubik",
    color: "#f1f1f1",
    style: [TextStyle.BOLD],
    transformation: "uppercase",
    horizontalAlignment: "center",

    // 2 text
    fontSize: 2.0,
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
    color: "#f1f1f1",
    style: [TextStyle.BOLD],
    // transformation: "uppercase",
    horizontalAlignment: "center",

    // 2 text
    fontSize: 2.65,
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
    color: "#aaaaaa",
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
};
