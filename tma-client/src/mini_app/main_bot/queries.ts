import type { MainPageProps, Bot } from "./types";

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

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
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

      const data = await response.json();
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
          //todo remove
          "ngrok-skip-browser-warning": "",
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
}

const api = new ApiClient(
  "https://advanced-oddly-herring.ngrok-free.app/stardonationservice"
);

export const getControlledBots = api.getControlledBots.bind(api);
export const addBot = api.addBot.bind(api);
export const getAvatar = api.getAvatar.bind(api);
