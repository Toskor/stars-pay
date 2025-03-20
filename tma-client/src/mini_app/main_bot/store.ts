import { get, writable, type Writable } from "svelte/store";
import type { MainPageProps } from "./types";
import { getControlledBots, getAvatar } from "./queries";

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

  try {
    const updatedData = { ...store.data };

    await Promise.all(
      updatedData.bots.map(async (bot) => {
        try {
          const botAvatar = await getAvatarAsObjectUrl(initData, bot.numeric_id.toString());
          if (botAvatar) {
            bot.avatar = botAvatar;
          }

          if (bot.owner && bot.owner.id) {
            const ownerAvatar = await getAvatarAsObjectUrl(
              initData,
              bot.owner.id.toString()
            );
            if (ownerAvatar) {
              bot.owner.avatarUrl = ownerAvatar;
            }
          }

          if (bot.admins && bot.admins.length > 0) {
            await Promise.all(
              bot.admins.map(async (admin) => {
                if (admin && admin.id) {
                  const adminAvatar = await getAvatarAsObjectUrl(
                    initData,
                    admin.id.toString()
                  );
                  if (adminAvatar) {
                    admin.avatarUrl = adminAvatar;
                  }
                }
              })
            );
          }
        } catch (error) {
          console.error(`Error updating avatars for bot ${bot.id}:`, error);
        }
      })
    );

    botsStore.update((store) => ({
      ...store,
      data: updatedData,
    }));

    console.log("All avatars updated successfully");
  } catch (error) {
    console.error("Error updating all avatars:", error);
  }
}

export async function getAvatarAsObjectUrl(
  initData: string,
  userId: string
): Promise<string | null> {
  try {
    const blob = await getAvatar(initData, userId);

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
