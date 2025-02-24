export type Page = "main" | "edit" | "create";

export interface MainPageProps {
    bots: Bot[];
    hasSuspendedBots: boolean;
  }

  export interface Bot {
    id: number;
    name: string;
    avatar: string;

    userRole: "owner" | "admin" | "user";
    owner: User;
    admins: User[];

    suspended?: boolean;
    balance?: number;
  }

  export type User = {
    id: number;
    //ex torsor
    username: string;
    //ex Григорий Борисов
    name: string;
    avatar_url: string;
  };