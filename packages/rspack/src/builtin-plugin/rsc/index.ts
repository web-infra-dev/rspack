import { Coordinator } from './Coordinator';
import {
  RscClientPlugin,
  type RscClientPluginOptions,
} from './RscClientPlugin';
import { RscServerPlugin } from './RscServerPlugin';

declare class ServerPlugin extends RscServerPlugin {
  constructor(options?: Omit<RscClientPluginOptions, 'coordinator'>);
}

declare class ClientPlugin extends RscClientPlugin {}

export const rsc = {
  createPlugins: (): {
    ServerPlugin: new (
      options?: Omit<RscClientPluginOptions, 'coordinator'>,
    ) => ServerPlugin;
    ClientPlugin: new () => ClientPlugin;
  } => {
    const coordinator = new Coordinator();

    return {
      ServerPlugin: class ServerPlugin extends RscServerPlugin {
        constructor(options: Omit<RscClientPluginOptions, 'coordinator'> = {}) {
          super({ coordinator, ...options });
        }
      },
      ClientPlugin: class ClientPlugin extends RscClientPlugin {
        constructor() {
          super({ coordinator });
        }
      },
    };
  },

  Layers: {
    /**
     * The layer for server-only runtime and picking up `react-server` export conditions.
     */
    rsc: 'react-server-components',
    /**
     * Server Side Rendering layer for app.
     */
    ssr: 'server-side-rendering',
  } as const,
};
