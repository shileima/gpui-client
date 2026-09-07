export interface GpuiConfig {
  userMis: string;
  workspaceRoot: string;
  aichatRemoteUrl: string;
  ncToken?: string;
}

export const applyGpuiAuth = (cfg: GpuiConfig) => {
  if (cfg.ncToken) {
    localStorage.setItem('nc-token', cfg.ncToken);
  }
};

const WORKSPACE_MEMORY_KEY = 'claude-code-last-workspace';

export const initDefaultWorkspace = (workspaceRoot: string) => {
  if (!workspaceRoot) return;
  try {
    const raw = localStorage.getItem(WORKSPACE_MEMORY_KEY);
    const parsed = raw ? JSON.parse(raw) : null;
    if (parsed?.workspacePath) return;
    localStorage.setItem(
      WORKSPACE_MEMORY_KEY,
      JSON.stringify({ environmentId: '', workspacePath: workspaceRoot }),
    );
    window.dispatchEvent(
      new CustomEvent('workspace-changed', {
        detail: { environmentId: '', workspacePath: workspaceRoot },
      }),
    );
  } catch {
    // ignore
  }
};

export const loadGpuiConfig = async (): Promise<GpuiConfig> => {
  const res = await fetch('/__gpui_bridge/config');
  const data = await res.json();
  window.__AICHAT_REMOTE_URL__ = data.aichatRemoteUrl;
  const cfg: GpuiConfig = {
    userMis: data.userMis,
    workspaceRoot: data.workspaceRoot,
    aichatRemoteUrl: data.aichatRemoteUrl,
    ncToken: data.ncToken,
  };
  window.__GPUI_CONFIG__ = cfg;
  applyGpuiAuth(cfg);
  return cfg;
};

export const getElectronAPI = () => window.electronAPI;

export const isGpuiClient = () => Boolean(window.__gpuiClient || window.electronAppMode?.isGpuiClient);

export const isElectronClient = () => Boolean(window.electronAPI || isGpuiClient());
