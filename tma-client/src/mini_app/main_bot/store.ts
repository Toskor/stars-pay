import { get, writable, type Writable } from "svelte/store";
import type { MainPageProps } from "./types";
import { getControlledBots, getAvatar } from "./queries";
import type { DonationButton } from "../stream_bot/types";

export type BotsStoreType = {
  isLoaded: boolean;
  isLoading: boolean;
  error: string | null;
  data: MainPageProps | null;
  loadTime: number | null;
};

export const botsStore: Writable<{
  isLoaded: boolean;
  isLoading: boolean;
  error: string | null;
  data: MainPageProps | null;
  loadTime: number | null;
}> = writable({
  isLoaded: false,
  isLoading: false,
  error: null,
  data: null,
  loadTime: null,
});

export function getDonationButtonsLen(botId: string): number {
  const store = get(botsStore);
  if (!store.data) return 0;
  return (
    store.data.bots.find((b) => b.id === botId)?.preview_data?.donation_buttons
      .length || 0
  );
}

export function addDonationButtonToBot(
  botId: string,
  newButtons: DonationButton[]
) {
  botsStore.update((store) => {
    if (!store.data) return store;

    const updatedBots = store.data.bots.map((bot) => {
      if (bot.id === botId) {
        const updatedPreviewData = {
          ...bot.preview_data,
          donation_buttons: [
            ...(bot.preview_data?.donation_buttons || []),
            ...newButtons,
          ],
        };

        return {
          ...bot,
          preview_data: updatedPreviewData,
        };
      }
      return bot;
    });

    return {
      ...store,
      data: {
        ...store.data,
        bots: updatedBots,
      },
    };
  });
}

export async function loadBotsData(initData: string): Promise<void> {
  const appStartTime = performance.now();

  try {
    botsStore.update((store) => ({ ...store, isLoading: true }));

    const result = await getControlledBots(initData);

    if (result.success) {
      const loadTime = performance.now() - appStartTime;
      console.log(`App load time: ${loadTime.toFixed(2)}ms`);

      botsStore.set({
        isLoaded: true,
        isLoading: false,
        error: null,
        data: result.data,
        loadTime,
      });

      await updateAllAvatars(initData);
    } else {
      botsStore.update((store) => ({
        ...store,
        isLoaded: true,
        isLoading: false,
        error: result.error,
      }));
    }
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    botsStore.update((store) => ({
      ...store,
      isLoaded: true,
      isLoading: false,
      error: errorMessage,
    }));
  }
}

export async function refreshBotsData(initData: string): Promise<boolean> {
  const startTime = performance.now();

  botsStore.update((store) => ({ ...store, isLoading: true }));

  try {
    const result = await getControlledBots(initData);

    if (result.success) {
      const loadTime = performance.now() - startTime;
      console.log(`Refresh load time: ${loadTime.toFixed(2)}ms`);

      botsStore.set({
        isLoaded: true,
        isLoading: false,
        error: null,
        data: result.data,
        loadTime,
      });

      await updateAllAvatars(initData);

      return true;
    } else {
      botsStore.update((store) => ({
        ...store,
        data: null,
        isLoaded: true,
        isLoading: false,
        error: result.error,
      }));

      return false;
    }
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    botsStore.update((store) => ({
      ...store,
      data: null,
      isLoaded: true,
      isLoading: false,
      error: errorMessage,
    }));

    return false;
  }
}

async function updateAllAvatars(initData: string): Promise<void> {
  const store = get(botsStore);
  if (!store.data) return;

  const updatedData = { ...store.data };

  updatedData.bots.forEach((bot) => {
    getAvatarAsObjectUrl(initData, bot.numeric_id.toString(), bot.id)
      .then((botAvatar) => {
        if (botAvatar) {
          bot.avatar = botAvatar;
          botsStore.update((store) => ({ ...store, data: updatedData }));
        }
      })
      .catch((error) =>
        console.error(`Error updating bot avatar ${bot.id}:`, error)
      );

    if (bot.owner && bot.owner.id) {
      getAvatarAsObjectUrl(initData, bot.owner.id.toString(), bot.id)
        .then((ownerAvatar) => {
          if (ownerAvatar) {
            bot.owner.avatarUrl = ownerAvatar;
            botsStore.update((store) => ({ ...store, data: updatedData }));
          }
        })
        .catch((error) =>
          console.error(`Error updating owner avatar ${bot.owner.id}:`, error)
        );
    }

    bot.admins.forEach((admin) => {
      if (admin && admin.id) {
        getAvatarAsObjectUrl(initData, admin.id.toString(), bot.id)
          .then((adminAvatar) => {
            if (adminAvatar) {
              admin.avatarUrl = adminAvatar;
              botsStore.update((store) => ({ ...store, data: updatedData }));
            }
          })
          .catch((error) =>
            console.error(`Error updating admin avatar ${admin.id}:`, error)
          );
      }
    });
  });
}

export async function getAvatarAsObjectUrl(
  initData: string,
  userId: string,
  bot_id: string
): Promise<string | null> {
  try {
    const blob = await getAvatar(initData, userId, bot_id);

    if (blob) {
      return URL.createObjectURL(blob);
    }

    return null;
  } catch (error) {
    console.error("Error getting avatar ", userId, " as object url:", error);
    return null;
  }
}

export function revokeAvatarObjectUrl(objectUrl: string | null): void {
  if (objectUrl) {
    URL.revokeObjectURL(objectUrl);
  }
}

export function revokeAllAvatarObjectUrls(): void {
  const store = get(botsStore);
  if (store.data) {
    store.data.bots.forEach((bot) => {
      if (bot.avatar) {
        revokeAvatarObjectUrl(bot.avatar);
      }

      if (bot.owner.avatarUrl) {
        revokeAvatarObjectUrl(bot.owner.avatarUrl);
      }

      bot.admins.forEach((admin) => {
        if (admin.avatarUrl) {
          revokeAvatarObjectUrl(admin.avatarUrl);
        }
      });
    });
  }
}
