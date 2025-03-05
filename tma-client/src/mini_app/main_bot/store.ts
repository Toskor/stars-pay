import { writable, type Writable } from "svelte/store";
import type { MainPageProps } from "./types";
import { getControlledBots } from "./queries";

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
      console.log("init data", result.data);

      botsStore.set({
        isLoaded: true,
        isLoading: false,
        error: null,
        data: result.data,
        loadTime,
      });
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
