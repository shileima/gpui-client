import { useEffect, useState } from 'react';
import { getElectronAPI } from '../utils/gpuiBridge';

export const DEFAULT_LOCAL_AIGC_PORT = 20318;

export interface LocalAigcRuntime {
  aigcBaseUrl: string;
  deviceId?: string;
  ready: boolean;
}

export function buildLocalCCDataMedium(
  aigcBaseUrl: string | undefined,
  sessionId?: string | null,
  opts?: {
    deviceId?: string;
    workDir?: string;
    agentName?: string;
    subAgentId?: string;
  },
) {
  const safeBaseUrl = aigcBaseUrl || `http://127.0.0.1:${DEFAULT_LOCAL_AIGC_PORT}`;

  const sessionParams: Record<string, unknown> = {
    codeProjectId: sessionId,
  };
  if (opts?.deviceId) sessionParams.deviceId = opts.deviceId;
  if (opts?.workDir) sessionParams.workDir = opts.workDir;
  if (opts?.agentName) sessionParams.agentName = opts.agentName;
  if (opts?.subAgentId) sessionParams.subAgentId = opts.subAgentId;

  return {
    chatAttrs: { source: 'codeAgent' },
    system: {
      provider: 'cclocal' as const,
      cclocal: {
        proxyUrl: safeBaseUrl,
        apiHost: safeBaseUrl,
        wsHost: safeBaseUrl.replace(/^http/, 'ws'),
        providerType: 'claudeCodeLocal',
      },
    },
    sessionParams,
  };
}

export function buildRemoteDataMedium(sessionId?: string | null, opts?: { spaceId?: string; ssoId?: string }) {
  return {
    chatAttrs: { source: 'gpuiClient' },
    system: {
      provider: 'bots' as const,
    },
    sessionParams: {
      codeProjectId: sessionId,
      spaceId: opts?.spaceId,
      ssoId: opts?.ssoId,
    },
  };
}

export function useLocalAigcRuntime(enabled: boolean): LocalAigcRuntime {
  const [runtime, setRuntime] = useState<LocalAigcRuntime>({
    aigcBaseUrl: `http://127.0.0.1:${DEFAULT_LOCAL_AIGC_PORT}`,
    ready: false,
  });

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;

    const refresh = async () => {
      const api = getElectronAPI();
      try {
        if (api?.aigcServiceStart) {
          await api.aigcServiceStart();
        }
        const status = api?.aigcServiceGetStatus ? await api.aigcServiceGetStatus() : null;
        const device = api?.aigcServiceGetLocalDeviceId ? await api.aigcServiceGetLocalDeviceId() : null;
        if (cancelled) return;
        setRuntime({
          aigcBaseUrl: status?.baseUrl || `http://127.0.0.1:${DEFAULT_LOCAL_AIGC_PORT}`,
          deviceId: device?.deviceId,
          ready: Boolean(status?.running),
        });
      } catch {
        if (!cancelled) {
          setRuntime((prev) => ({ ...prev, ready: false }));
        }
      }
    };

    void refresh();
    const timer = window.setInterval(refresh, 30000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [enabled]);

  return runtime;
}
