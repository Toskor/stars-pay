// Audio service for managing sound effects
class AudioService {
  private static instance: AudioService;
  private audioContext: AudioContext | null = null;
  private soundCache: Map<string, AudioBuffer> = new Map();
  //just test cdn
  private baseUrl = "https://advanced-oddly-herring.ngrok-free.app/sound/";

  private constructor() {}

  static getInstance(): AudioService {
    if (!AudioService.instance) {
      AudioService.instance = new AudioService();
    }
    return AudioService.instance;
  }

  async init() {
    if (!this.audioContext) {
      this.audioContext = new AudioContext();
    }
  }

  async loadSound(soundName: string): Promise<AudioBuffer> {
    if (!this.audioContext) {
      await this.init();
    }

    const cachedSound = this.soundCache.get(soundName);
    if (cachedSound) {
      return cachedSound;
    }

    try {
      const response = await fetch(`${this.baseUrl}${soundName}`);
      const arrayBuffer = await response.arrayBuffer();
      const audioBuffer = await this.audioContext!.decodeAudioData(arrayBuffer);

      this.soundCache.set(soundName, audioBuffer);
      return audioBuffer;
    } catch (error) {
      console.error(`Failed to load sound ${soundName}:`, error);
      throw error;
    }
  }

  async playSound(soundName: string) {
    try {
      const buffer = await this.loadSound(soundName);
      if (!this.audioContext) return;

      const source = this.audioContext.createBufferSource();
      source.buffer = buffer;
      source.connect(this.audioContext.destination);
      source.start();
    } catch (error) {
      console.error(`Failed to play sound ${soundName}:`, error);
    }
  }

  async preloadSounds(soundNames: string[]) {
    await Promise.all(soundNames.map((name) => this.loadSound(name)));
  }

  clearCache() {
    this.soundCache.clear();
  }
}

export const audioService = AudioService.getInstance();
