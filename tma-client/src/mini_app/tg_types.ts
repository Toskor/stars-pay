export type Button = {
  name: string;
  invoice_url: string;
};

export type AppConfig = {
  header_text: string;
  buttons: Button[];
  api_url: string; // .../bot_id
  page_description: string;
  owner: number;
  admins: number[];
};

export type User = {
  id: number;
  username: string;
  avatar_url: string;
};

export type Bot = {
  id: string;
  controll_type: "owner" | "admin";
  username: string;
  owner: User;
  admins: User[];
  avatar_url: string;
};

export type ControlledBots = {
  bots: Bot[];
};
