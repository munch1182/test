declare global {
  interface Window {
    bridge: {
      send<T>(command: string, payload: any | undefined): Promise<T>;
    };
  }
}
