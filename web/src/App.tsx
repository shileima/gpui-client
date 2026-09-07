import React, { useEffect, useState } from 'react';
import { ConfigProvider, Spin, message } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import { ChatHome } from './views/ChatHome';
import { initDefaultWorkspace, loadGpuiConfig } from './utils/gpuiBridge';
import './styles.css';

const App: React.FC = () => {
  const [ready, setReady] = useState(false);
  const [userMis, setUserMis] = useState('gpui-user');
  const [workspaceRoot, setWorkspaceRoot] = useState('');

  useEffect(() => {
    const boot = async () => {
      try {
        const cfg = await loadGpuiConfig();
        initDefaultWorkspace(cfg.workspaceRoot);
        setUserMis(cfg.userMis);
        setWorkspaceRoot(cfg.workspaceRoot);
      } catch (error) {
        console.warn('加载 GPUI 配置失败，使用默认值', error);
        message.warning('未能连接 GPUI Bridge，部分本地能力不可用');
      } finally {
        setReady(true);
      }
    };
    void boot();
  }, []);

  if (!ready) {
    return (
      <div className="app-loading">
        <Spin size="large" tip="正在初始化 AI 聊天..." />
      </div>
    );
  }

  return (
    <ConfigProvider locale={zhCN}>
      <ChatHome userMis={userMis} defaultWorkDir={workspaceRoot} />
    </ConfigProvider>
  );
};

export default App;
