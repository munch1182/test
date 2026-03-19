// This file is auto-generated. Do not edit manually.


declare global {
  interface Window {
    bridge: {
      send<T>(command: string, payload: any | undefined): Promise<T>;
    };
  }
}


export const commands = {
    list_plugins: (): Promise<null> => window.bridge.send<null>('list_plugins', undefined),
};

