import type { RspackPluginInstance } from '../..';
import { Coordinator } from './Coordinator';
import {
  RscClientPlugin,
  type RscClientPluginOptions,
} from './RscClientPlugin';
import { RscServerPlugin } from './RscServerPlugin';

export function createRscPlugins(): {
  ServerPlugin: RspackPluginInstance;
  ClientPlugin: RspackPluginInstance;
} {
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
}

export const RSC_LAYERS_NAMES = {
  /**
   * The layer for server-only runtime and picking up `react-server` export conditions.
   */
  REACT_SERVER_COMPONENTS: 'react-server-components',
  /**
   * Server Side Rendering layer for app.
   */
  SERVER_SIDE_RENDERING: 'server-side-rendering',
};
