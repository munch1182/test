// This file is auto-generated. Do not edit manually.

export interface ScanParam {
    path: string;
    load_exists: boolean;
}

export interface ScanResult {
    loaded: string[];
    failds: ScanFailItem[];
    ignores: string[];
}

export interface Plugin {
    id: string;
    name: string;
    version: string;
    url: string;
}

export interface ScanFailItem {
    url: string;
    path: string;
    reason: string;
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
	/**
	 * 
	 * * 扫描指定位置的插件
	 */
	scan_plugins: (args: { p: ScanParam }): Promise<ScanResult> => window.bridge.send<ScanResult>('scan_plugins', args),
};

