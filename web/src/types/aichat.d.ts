declare module 'aichat/dialog' {
  import type { ComponentType } from 'react';
  export const PureDialog: ComponentType<any>;
}

declare module 'aichat/chatExt' {
  import type { ComponentType } from 'react';
  const AichatModule: ComponentType<any>;
  export default AichatModule;
}
