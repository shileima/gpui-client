import axios from 'axios';
import { getElectronAPI } from '../utils/gpuiBridge';

export const FALLBACK_CODE_EXPERT_AGENT_ID = 'agent-7a6e76a5-7';

const DEFAULT_GATEWAY_HOST = 'https://digitalgateway.sankuai.com/platform';
const CHAT_DATA_SOURCE = 'automan';

export const getGatewayHost = () =>
  import.meta.env.VITE_GATEWAY_HOST || DEFAULT_GATEWAY_HOST;

const authHeaders = () => ({
  'Content-Type': 'application/json',
  'xc-auth': localStorage.getItem('nc-token') || '',
});

export const generateChatId = () =>
  typeof crypto !== 'undefined' && crypto.randomUUID
    ? crypto.randomUUID().replace(/-/g, '')
    : `${Date.now()}${Math.random().toString(36).slice(2, 10)}`;

export const extractSubmitText = (payload: Record<string, unknown>): string => {
  const newMsg = (payload.newMsg || {}) as Record<string, unknown>;
  const rawContent = newMsg.content ?? payload.content ?? payload.input ?? payload.value;
  if (typeof rawContent === 'string') return rawContent.trim();
  if (rawContent && typeof rawContent === 'object') {
    const obj = rawContent as Record<string, unknown>;
    return String(obj.title || obj.content || '').trim();
  }
  return String(rawContent || '').trim();
};

export const formatSubmitContent = (value: unknown) => {
  if (!value || typeof value === 'string') return { content: String(value || '') };
  if (typeof value !== 'object') return { content: '' };
  const obj = value as Record<string, unknown>;
  return {
    content: String(obj.title || obj.content || ''),
    ...(obj.imgUrl ? { imgUrl: obj.imgUrl } : {}),
    ...(obj.filePath ? { files: obj.filePath } : {}),
  };
};

export async function createLocalCcSession(aigcBaseUrl: string, params: Record<string, unknown>) {
  const api = getElectronAPI();
  if (api?.aigcSessionCreateV2) {
    return api.aigcSessionCreateV2(params);
  }
  const res = await axios.post(`${aigcBaseUrl}/open/v1/human/machine/newSession`, params, { timeout: 30000 });
  return res?.data;
}

export async function sendLocalCcInitContent(proxyUrl: string, data: Record<string, unknown>) {
  const res = await axios.post(`${proxyUrl}/open/v1/human/machine/asynInvokeChat`, data, { timeout: 30000 });
  return res?.data;
}

export async function queryLocalAgentAiConfig(proxyUrl: string, uuid: string, userLevel = true) {
  const res = await axios.get(`${proxyUrl}/api/agent/queryAgentAiConfig`, {
    params: { uuid, userLevel },
    timeout: 15000,
  });
  return res?.data;
}

export async function queryRemoteAgentAiConfig(agentUuid: string) {
  const gateway = getGatewayHost();
  const res = await axios.get(`${gateway}/api/agent/queryAgentAiConfig`, {
    params: { uuid: agentUuid, userLevel: true },
    timeout: 15000,
    headers: authHeaders(),
  });
  return res?.data;
}

export async function createRemoteSession(params: {
  sessionId: string;
  subAgentId: string;
  operator: string;
  name?: string;
}) {
  const gateway = getGatewayHost();
  const res = await axios.post(
    `${gateway}/open/v1/human/machine/newSession`,
    {
      sessionId: params.sessionId,
      subAgentId: params.subAgentId,
      operator: params.operator,
      name: params.name || '新的聊天',
    },
    { timeout: 30000, headers: authHeaders() },
  );
  return res?.data;
}

export async function sendRemoteInitContent(params: {
  sessionId: string;
  subAgentId: string;
  operator: string;
  content: unknown;
  messageId: string;
}) {
  const gateway = getGatewayHost();
  const res = await axios.post(
    `${gateway}/open/v1/human/machine/asynInvokeChat`,
    {
      source: CHAT_DATA_SOURCE,
      subAgentId: params.subAgentId,
      sessionId: params.sessionId,
      operator: params.operator,
      fillParams: false,
      reGenerate: false,
      extra: {},
      system: {
        xc_auth: localStorage.getItem('nc-token'),
        lastUdId: '8fd92bb5',
      },
      message: {
        role: 'user',
        ...formatSubmitContent(params.content),
        messageId: params.messageId,
      },
      messageId: params.messageId,
    },
    { timeout: 30000, headers: authHeaders() },
  );
  return res?.data;
}
