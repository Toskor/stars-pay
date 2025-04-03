import { writable } from "svelte/store";
import type { WSMessage } from "./ws";

export interface DonationItem {
  id: string;
  //username
  from: string;
  total_amount: number;
  //mb url for image
  invoice_payload: string;

  url: string;
  isGif: boolean;
  timestamp: number;
}

interface DonationState {
  queue: DonationItem[];
  isDisplayEnabled: boolean;
}

function createDonationStore() {
  const { subscribe, set, update } = writable<DonationState>({
    queue: [],
    isDisplayEnabled: true,
  });

  return {
    subscribe,
    addToQueue: (ws_msg: WSMessage) => {
      update((state) => {
        if (!ws_msg.ok) {
          return state;
        }
        
        const isGif = ws_msg.invoice_payload.toLowerCase().endsWith(".gif");
        const newItem: DonationItem = {
          id: crypto.randomUUID(),
          url: ws_msg.invoice_payload,
          isGif,
          timestamp: Date.now(),
          from: ws_msg.from,
          total_amount: ws_msg.total_amount,
          invoice_payload: ws_msg.invoice_payload,
        };
        return {
          ...state,
          queue: [...state.queue, newItem],
        };
      });
    },
    removeFromQueue: (id: string) => {
      update((state) => ({
        ...state,
        queue: state.queue.filter((item) => item.id !== id),
      }));
    },
    clearQueue: () => {
      update((state) => ({
        ...state,
        queue: [],
      }));
    },
    toggleDisplay: () => {
      update((state) => ({
        ...state,
        isDisplayEnabled: !state.isDisplayEnabled,
      }));
    },
  };
}

export const donationStore = createDonationStore();
