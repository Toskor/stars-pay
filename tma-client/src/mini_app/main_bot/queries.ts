import type { PreviewData } from "../stream_bot/types";
import type { MainPageProps, Bot, User } from "./types";

type ApiResponse<T> =
  | {
      success: true;
      data: T;
    }
  | {
      success: false;
      error: string;
    };

class ApiClient {
  private baseUrl: string;
  private mainBotId: string;

  constructor(baseUrl: string, defaultBotId: string) {
    this.baseUrl = baseUrl;
    this.mainBotId = defaultBotId;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
    botId?: string
  ): Promise<ApiResponse<T>> {
    const targetBotId = botId || this.mainBotId;
    const requestUrl = `${this.baseUrl}/${targetBotId}${endpoint}`;

    try {
      const response = await fetch(requestUrl, {
        ...options,
        headers: {
          "Content-Type": "application/json;charset=utf-8",
          //todo remove
          "ngrok-skip-browser-warning": "",
          ...options.headers,
        },
      });

      if (!response.ok) {
        const errorText = await response.text();
        console.error(errorText);
        return {
          success: false,
          error: errorText,
        };
      }

      const data: T = await response.json();
      return {
        success: true,
        data,
      };
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Unknown error";
      console.error(errorMessage);
      return {
        success: false,
        error: errorMessage,
      };
    }
  }

  async getControlledBots(
    initData: string
  ): Promise<ApiResponse<MainPageProps>> {
    return this.request<MainPageProps>("/controlledBots", {
      method: "GET",
      headers: {
        "X-Telegram-InitData": initData,
      },
    });
  }

  async addBot(
    initData: string,
    botToken: string
  ): Promise<ApiResponse<{ status: string; bot_data: Bot }>> {
    return this.request<{ status: string; bot_data: Bot }>("/addBot", {
      method: "POST",
      headers: {
        "X-Telegram-InitData": initData,
      },
      body: JSON.stringify({ bot_token: botToken }),
    });
  }

  async getAvatar(initData: string, userId: string): Promise<Blob | null> {
    try {
      const response = await fetch(`${this.baseUrl}/avatar/${userId}`, {
        method: "GET",
        headers: {
          "X-Telegram-InitData": initData,
        },
      });

      if (!response.ok) {
        console.error(`Failed to fetch avatar: ${response.status}`);
        return null;
      }

      const contentType = response.headers.get("content-type");
      if (contentType !== "image/jpeg") {
        console.error(`Unexpected content type: ${contentType}`);
        return null;
      }

      return await response.blob();
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Unknown error";
      console.error(`Avatar fetch error: ${errorMessage}`);
      return null;
    }
  }

  async removeBotAdmin(
    initData: string,
    botId: string,
    adminId: number
  ): Promise<ApiResponse<{ status: string }>> {
    return this.request<{ status: string }>("/removeBotAdmin", {
      method: "POST",
      headers: {
        "X-Telegram-InitData": initData,
      },
      body: JSON.stringify({ bot_id: botId, admin_id: adminId }),
    });
  }

  async removeBot(
    initData: string,
    botId: string
  ): Promise<ApiResponse<{ status: string }>> {
    return this.request<{ status: string }>("/removeBot", {
      method: "POST",
      headers: {
        "X-Telegram-InitData": initData,
      },
      body: JSON.stringify({ bot_id: botId }),
    });
  }

  async changeBotToken(
    initData: string,
    botId: string,
    newToken: string
  ): Promise<ApiResponse<{ status: string }>> {
    return this.request<{ status: string }>("/changeBotToken", {
      method: "POST",
      headers: {
        "X-Telegram-InitData": initData,
      },
      body: JSON.stringify({ bot_id: botId, new_token: newToken }),
    });
  }

  async addAdmin(
    initData: string,
    botId: string,
    adminId: number
  ): Promise<
    ApiResponse<{
      status: string;
      admin_info: User;
    }>
  > {
    return this.request<{
      status: string;
      admin_info: User;
    }>("/addBotAdmin", {
      method: "POST",
      headers: {
        "X-Telegram-InitData": initData,
      },
      body: JSON.stringify({ bot_id: botId, admin_id: adminId }),
    });
  }

  async getConfig(
    initData: string,
    bot_id: string
  ): Promise<ApiResponse<PreviewData>> {
    return this.request<PreviewData>("/config", {
      method: "POST",
      headers: {
        "X-Telegram-InitData": initData,
      },
      body: JSON.stringify({ target_bot_id: bot_id }),
    });
  }

  async updateConfig(
    initData: string,
    bot_id: string,
    config: string
  ): Promise<ApiResponse<{ status: string }>> {
    return this.request<{ status: string }>("/updateConfig", {
      method: "POST",
      headers: {
        "X-Telegram-InitData": initData,
      },
      body: JSON.stringify({ target_bot_id: bot_id, app_config: config }),
    });
  }
}

const api = new ApiClient(
  "https://advanced-oddly-herring.ngrok-free.app",
  "stardonationservice"
);

export const getControlledBots = api.getControlledBots.bind(api);
export const addAdmin = api.addAdmin.bind(api);
export const getAvatar = api.getAvatar.bind(api);
export const removeBotAdmin = api.removeBotAdmin.bind(api);
export const removeBot = api.removeBot.bind(api);
export const changeBotToken = api.changeBotToken.bind(api);
export const addBot = api.addBot.bind(api);
export const getConfig = api.getConfig.bind(api);
export const updateConfig = api.updateConfig.bind(api);
