/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_GATEWAY_HOST?: string;
  readonly VITE_DEFAULT_AGENT_UUID?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module 'aichat/dialog' {
  import type { ComponentType } from 'react';
  export const PureDialog: ComponentType<any>;
}

declare module 'aichat/chatExt' {
  import type { ComponentType } from 'react';
  const AichatModule: ComponentType<any>;
  export default AichatModule;
}

declare global {
  interface Window {
    electronAPI?: Record<string, (...args: any[]) => Promise<any>>;
    electronAppMode?: { isElectron?: boolean; isGpuiClient?: boolean };
    __AICHAT_REMOTE_URL__?: string;
    __GPUI_CONFIG__?: {
      userMis?: string;
      workspaceRoot?: string;
      aichatRemoteUrl?: string;
    };
    __gpuiClient?: boolean;
  }
}

export {};
