// This file is auto-generated. Do not edit manually.

export interface Plugin {
    id: string;
    name: string;
    version: string;
}


declare global {
  interface Window {
    bridge: {
      send<T>(command: string, payload: any | undefined): Promise<T>;
    };
  }
}


export const commands = {
	/**
	 * 
	 * * 返回所有的插件信息
	 */
	list_plugins: (): Promise<Plugin[]> => window.bridge.send<Plugin[]>('list_plugins', undefined),
	/**
	 * 
	 * * 调用插件
	 */
	call: (args: { id: string }): Promise<string> => window.bridge.send<string>('call', args),
};

