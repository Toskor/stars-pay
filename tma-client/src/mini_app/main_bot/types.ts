export type Page = "main" | "edit" | "create";

export interface MainPageProps {
  bots: Bot[];
}

export interface Bot {
  id: string;
  name: string;
  avatar: string;

  userRole: "owner" | "admin" | "user";
  owner: User;
  admins: User[];

  suspended?: boolean;
  debt?: number;
}

export type User = {
  id: number;
  //ex torsor
  username: string;
  //ex Григорий Борисов
  name: string;
  avatar_url: string;
};

export const testBots: Bot[] = [
  {
    id: "1",
    name: "YomlDevBot",
    avatar: "https://avatars.githubusercontent.com/u/84640980?v=4",
    userRole: "owner",
    owner: {
      id: 1,
      username: "YomlDevBot",
      name: "YomlDevBot",
      avatar_url: "https://avatars.githubusercontent.com/u/84640980?v=4",
    },
    admins: [],
    suspended: true,
    debt: 100,
  },
  {
    id: "2",
    name: "YomlDevBot2",
    avatar: "https://avatars.githubusercontent.com/u/84640980?v=4",
    userRole: "owner",
    owner: {
      id: 1,
      username: "YomlDevBot",
      name: "YomlDevBot",
      avatar_url: "https://avatars.githubusercontent.com/u/84640980?v=4",
    },
    admins: [],
  },
];
