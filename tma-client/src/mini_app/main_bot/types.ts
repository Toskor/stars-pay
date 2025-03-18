import type { PreviewData } from "../stream_bot/types";

export type Page =
  | "main"
  | "edit"
  | "create"
  | "change_token"
  | "manage_donation_buttons"
  | "preview_stream_bot"
  | "add_admin";

export interface MainPageProps {
  bots: Bot[];
}

export interface Bot {
  id: string;
  numeric_id: number;
  name: string;
  avatar?: string;

  userRole: "owner" | "admin" | "user";
  owner: User;
  admins: User[];

  suspended?: boolean;
  debt?: number;

  preview_data?: PreviewData;
}

export type User = {
  id: number;
  //ex torsor
  username: string;
  //ex Григорий Борисов
  name: string;
  avatarUrl?: string;
};

export const testBots: Bot[] = [
  {
    id: "1",
    numeric_id: 1,
    name: "YomlDevBot",
    avatar: "https://avatars.githubusercontent.com/u/84640980?v=4",
    userRole: "owner",
    owner: {
      id: 1,
      username: "YomlDevBot",
      name: "YomlDevBot",
      avatarUrl: "https://avatars.githubusercontent.com/u/84640980?v=4",
    },
    admins: [],
    suspended: true,
    debt: 100,
  },
  {
    id: "2",
    numeric_id: 2,
    name: "YomlDevBot2",
    avatar: "https://avatars.githubusercontent.com/u/84640980?v=4",
    userRole: "owner",
    owner: {
      id: 1,
      username: "YomlDevBot",
      name: "YomlDevBot",
      avatarUrl: "https://avatars.githubusercontent.com/u/84640980?v=4",
    },
    admins: [],
  },
];
