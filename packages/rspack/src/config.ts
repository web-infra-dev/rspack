import type * as Config from './config/index';
import type { RawOptions } from '@rspack/binding';

import { createRspackModuleRuleAdapter } from './server';

export interface RspackOptions {
  /**
   * Entry points of compilation.
   */
  entry?: Config.Entry;
  /**
   * An **absolute** path pointed the root directory.
   */
  context?: Config.Context;
  /**
   * An array of plugins
   */
  plugins?: Config.Plugins;
  /**
   * dev server
   */
  dev?: Config.Dev;
  /**
   * Module configuration.
   */
  module?: Config.Module;
}

export function normalizePlugins(plugins: Config.Plugins) {
  return plugins.map((plugin) => {
    if (typeof plugin === 'string') {
      return [plugin];
    }
  });
}

export function User2Native(config: RspackOptions): RawOptions {
  return {
    entry: config.entry ?? {},
    context: config.context,
    plugins: normalizePlugins(config.plugins),
    module: {
      // TODO: support mutliple rules to support `Module Type`
      rules: (config.module.rules || []).map((rule) => {
        return {
          ...rule,
          uses: [
            createRspackModuleRuleAdapter({
              loaders: rule.uses,
            }),
          ],
        };
      }),
    },
  };
}
