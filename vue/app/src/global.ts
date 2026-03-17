declare global {
  interface Window {
    ipc: {
      postMessage<P, R>(channel: P): R | undefined;
    };
  }
}