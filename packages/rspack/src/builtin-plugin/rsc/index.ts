import { Coordinator } from './Coordinator';
import { RscClientPlugin } from './RscClientPlugin';
import {
  RscServerPlugin,
  type RscServerPluginOptions,
} from './RscServerPlugin';

declare class ServerPlugin extends RscServerPlugin {
  constructor(options?: Omit<RscServerPluginOptions, 'coordinator'>);
}

declare class ClientPlugin extends RscClientPlugin {}

export const rsc = {
  createPlugins: (): {
    ServerPlugin: new (
      options?: Omit<RscServerPluginOptions, 'coordinator'>,
    ) => ServerPlugin;
    ClientPlugin: new () => ClientPlugin;
  } => {
    const coordinator = new Coordinator();

    return {
      ServerPlugin: class ServerPlugin extends RscServerPlugin {
        constructor(options: Omit<RscServerPluginOptions, 'coordinator'> = {}) {
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
