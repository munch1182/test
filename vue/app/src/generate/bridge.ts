// This file is auto-generated. Do not edit manually.


declare global {
  interface Window {
    bridge: {
      send<T>(command: string, payload: any | undefined): Promise<T>;
    };
  }
}


export const commands = {
    list_plugins: (): Promise<any[]> => window.bridge.send<any[]>('list_plugins', undefined),
    call: (args: { id: string }): Promise<string> => window.bridge.send<string>('call', args),
};

