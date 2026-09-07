import React, { Suspense, lazy, useCallback, useMemo, useState } from 'react';
import { message, Spin, Tabs } from 'antd';
import {
  createLocalCcSession,
  createRemoteSession,
  extractSubmitText,
  FALLBACK_CODE_EXPERT_AGENT_ID,
  generateChatId,
  queryLocalAgentAiConfig,
  queryRemoteAgentAiConfig,
  sendLocalCcInitContent,
  sendRemoteInitContent,
} from '../api/chat';
import { buildLocalCCDataMedium, buildRemoteDataMedium, useLocalAigcRuntime } from '../hooks/useLocalAigcRuntime';
import { getElectronAPI, isGpuiClient } from '../utils/gpuiBridge';

const PureDialog = lazy(() =>
  import('aichat/dialog').then((m) => ({ default: m.PureDialog as React.ComponentType<any> })),
);
const AichatModule = lazy(() => import('aichat/chatExt'));

export type ChatMode = 'local' | 'remote';

export interface EmbeddedSession {
  sessionId: string;
  agentId: string;
  workDir?: string;
  model?: string;
  mode: ChatMode;
}

interface ChatHomeProps {
  userMis: string;
  defaultWorkDir: string;
}

const DEFAULT_CC_AGENT_NAME = 'main';
const WORKSPACE_MEMORY_KEY = 'claude-code-last-workspace';

const readStoredWorkDir = (fallback: string): string => {
  try {
    const raw = localStorage.getItem(WORKSPACE_MEMORY_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (typeof parsed?.workspacePath === 'string' && parsed.workspacePath.trim()) {
        return parsed.workspacePath.trim();
      }
    }
  } catch {
    // ignore
  }
  return fallback;
};

export const ChatHome: React.FC<ChatHomeProps> = ({ userMis, defaultWorkDir }) => {
  const [mode, setMode] = useState<ChatMode>('local');
  const [embeddedSession, setEmbeddedSession] = useState<EmbeddedSession | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const localRuntime = useLocalAigcRuntime(mode === 'local');

  const workDir = useMemo(
    () => readStoredWorkDir(defaultWorkDir || `${defaultWorkDir}/${DEFAULT_CC_AGENT_NAME}`),
    [defaultWorkDir],
  );

  const isLocalMode = mode === 'local';

  const handleLocalSubmit = useCallback(
    async (payload: Record<string, unknown>) => {
      if (!localRuntime.ready) {
        message.warning('本地 AIGC 未就绪，请确认 Automan/Electron 已启动或稍后重试');
        return;
      }

      setSubmitting(true);
      try {
        const content = extractSubmitText(payload);
        if (!content) return;

        const sessionParams = {
          operator: userMis,
          subAgentId: FALLBACK_CODE_EXPERT_AGENT_ID,
          agentUuid: FALLBACK_CODE_EXPERT_AGENT_ID,
          name: content.slice(0, 40),
          sessionType: 1,
          codeAgentMode: true,
          profileName: DEFAULT_CC_AGENT_NAME,
          workDir,
        };

        const sessionResp = await createLocalCcSession(localRuntime.aigcBaseUrl, sessionParams);
        const sessionId = sessionResp?.data?.sessionId || sessionResp?.sessionId;
        if (!sessionId) {
          throw new Error('创建会话失败');
        }

        await sendLocalCcInitContent(localRuntime.aigcBaseUrl, {
          sessionId,
          subAgentId: FALLBACK_CODE_EXPERT_AGENT_ID,
          content,
          operator: userMis,
          workDir,
          profileName: DEFAULT_CC_AGENT_NAME,
        });

        setEmbeddedSession({
          sessionId,
          agentId: FALLBACK_CODE_EXPERT_AGENT_ID,
          workDir,
          mode: 'local',
        });
      } catch (error) {
        console.error(error);
        message.error(error instanceof Error ? error.message : '发送失败');
      } finally {
        setSubmitting(false);
      }
    },
    [localRuntime.aigcBaseUrl, localRuntime.ready, userMis, workDir],
  );

  const handleRemoteSubmit = useCallback(
    async (payload: Record<string, unknown>) => {
      setSubmitting(true);
      try {
        const content = extractSubmitText(payload);
        if (!content) return;

        const agentUuid = import.meta.env.VITE_DEFAULT_AGENT_UUID || FALLBACK_CODE_EXPERT_AGENT_ID;
        const sessionId = generateChatId();
        const messageId = generateChatId();
        const newMsg = (payload.newMsg || {}) as Record<string, unknown>;

        const sessionResp = await createRemoteSession({
          sessionId,
          subAgentId: agentUuid,
          operator: userMis,
          name: content.slice(0, 40),
        });
        if (sessionResp?.code !== 0) {
          throw new Error(sessionResp?.msg || sessionResp?.message || '远端创建会话失败');
        }

        const initResp = await sendRemoteInitContent({
          sessionId,
          subAgentId: agentUuid,
          operator: userMis,
          content: newMsg.content ?? content,
          messageId,
        });
        if (initResp?.code !== 0) {
          throw new Error(initResp?.msg || initResp?.message || '远端发送消息失败');
        }

        setEmbeddedSession({ sessionId, agentId: agentUuid, mode: 'remote' });
      } catch (error) {
        console.error(error);
        message.error(error instanceof Error ? error.message : '远端发送失败');
      } finally {
        setSubmitting(false);
      }
    },
    [userMis],
  );

  const handleSubmit = mode === 'local' ? handleLocalSubmit : handleRemoteSubmit;

  if (embeddedSession) {
    return (
      <Suspense fallback={<Spin size="large" />}>
        <ChatDetailView
          session={embeddedSession}
          userMis={userMis}
          localRuntime={localRuntime}
          onBack={() => setEmbeddedSession(null)}
        />
      </Suspense>
    );
  }

  return (
    <div className="chat-home">
      <header className="chat-home__header">
        <div>
          <h1>GPUI AI 聊天</h1>
          <p>{isGpuiClient() ? '本地 Claude Code + 远端智能体' : 'Web 调试模式'}</p>
        </div>
        <Tabs
          activeKey={mode}
          onChange={(key) => setMode(key as ChatMode)}
          items={[
            { key: 'local', label: '本地 AIGC' },
            { key: 'remote', label: '远端 AIChat' },
          ]}
        />
      </header>

      <div className="chat-home__main">
        <div className="chat-home__compose">
          <Suspense fallback={<Spin size="large" />}>
            <PureDialog
              key={mode}
              mis={userMis}
              className="chat-home__dialog"
              autoFocus
              dialogType="mentions"
              autoSize={{ minRows: 2, maxRows: 8 }}
              loading={submitting}
              onSubmit={handleSubmit}
              hiddenOptions={isLocalMode ? ['environmentSelect'] : undefined}
              dataMedium={
                isLocalMode
                  ? buildLocalCCDataMedium(localRuntime.aigcBaseUrl, null, {
                      deviceId: localRuntime.deviceId,
                      workDir,
                      agentName: DEFAULT_CC_AGENT_NAME,
                      subAgentId: FALLBACK_CODE_EXPERT_AGENT_ID,
                    })
                  : buildRemoteDataMedium(null)
              }
              environment={{
                show: isLocalMode,
                localExecutionEnabled: isLocalMode,
                optimistic: isLocalMode,
                setDefaultWorkspacePath: isLocalMode,
              }}
              context={{
                canOpenWorkspaceFile: Boolean(getElectronAPI()),
              }}
              model={{
                show: isLocalMode,
                default: 'Auto',
              }}
            />
          </Suspense>
        </div>
      </div>

      <footer className="chat-home__status">
        {mode === 'local' ? (
          <span>
            本地 AIGC: {localRuntime.ready ? localRuntime.aigcBaseUrl : '未连接'}
            {localRuntime.deviceId ? ` · device ${localRuntime.deviceId.slice(0, 8)}` : ''}
          </span>
        ) : (
          <span>远端模式 · SSO 已配置</span>
        )}
      </footer>
    </div>
  );
};

interface ChatDetailViewProps {
  session: EmbeddedSession;
  userMis: string;
  localRuntime: ReturnType<typeof useLocalAigcRuntime>;
  onBack: () => void;
}

const ChatDetailView: React.FC<ChatDetailViewProps> = ({ session, userMis, localRuntime, onBack }) => {
  const [agentConfig, setAgentConfig] = useState<Record<string, unknown> | null>(null);

  React.useEffect(() => {
    const load = async () => {
      try {
        if (session.mode === 'local' && localRuntime.aigcBaseUrl) {
          const resp = await queryLocalAgentAiConfig(localRuntime.aigcBaseUrl, session.agentId);
          setAgentConfig(resp?.data || resp);
        } else {
          const resp = await queryRemoteAgentAiConfig(session.agentId);
          setAgentConfig(resp?.data || resp);
        }
      } catch (error) {
        console.warn('加载 agent 配置失败', error);
      }
    };
    void load();
  }, [session.agentId, session.mode, localRuntime.aigcBaseUrl]);

  const dataMedium =
    session.mode === 'local'
      ? buildLocalCCDataMedium(localRuntime.aigcBaseUrl, session.sessionId, {
          deviceId: localRuntime.deviceId,
          workDir: session.workDir,
          agentName: DEFAULT_CC_AGENT_NAME,
          subAgentId: session.agentId,
        })
      : buildRemoteDataMedium(session.sessionId, { ssoId: userMis });

  return (
    <div className="chat-detail">
      <div className="chat-detail__toolbar">
        <button type="button" onClick={onBack}>
          ← 新对话
        </button>
        <span>{session.mode === 'local' ? '本地 Claude Code' : '远端 AIChat'}</span>
      </div>
      <div className="chat-detail__body">
        <div className="chat-detail__module">
          <AichatModule
            mis={userMis}
            agentUuid={session.agentId}
            initStrategy="specifiedSession"
            initialSessionId={session.sessionId}
            uiMode="desktop"
            wrapperStyle={{ height: '100%' }}
            dataMedium={dataMedium}
            agentConfig={agentConfig || undefined}
            config={{
              header: {
                showTitle: 'session',
                showOptions: true,
                options: ['newChat', 'history'],
              },
              footer: {
                canOpenWorkspaceFile: Boolean(getElectronAPI()),
                claudeAgent: {
                  model: session.model || 'Auto',
                },
              },
              content: {
                addActionList: ['fork'],
              },
            }}
          />
        </div>
      </div>
    </div>
  );
};
