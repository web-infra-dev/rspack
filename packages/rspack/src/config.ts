import type { RawOptions } from "@rspack/binding";

import type { ModuleRule } from ".";
import { createRawModuleRuleUses } from ".";

export type Plugin = string | [string] | [string, unknown];

export interface RspackOptions {
  /**
   * Entry points of compilation.
   */
  entry?: RawOptions['entry'];
  /**
   * An **absolute** path pointed the
   */
  context?: RawOptions['context'];
  /**
   * An array of plugins
   */
  plugins?: Plugin[];
  /**
   * dev server
   */
  dev?: {
    port?: Number;
    static?: {
      directory?: string;
    };
  };
  /**
   * Module configuration.
   */
  module?: {
    rules?: ModuleRule[];
    parser?: RawOptions['module']['parser'];
  };
  define?: RawOptions['define'];
  target?: RawOptions['target'];
  mode?: RawOptions['mode'];
  external?: RawOptions['external'];
}

export function normalizePlugins(plugins: Plugin[]) {
  return plugins.map(plugin => {
    if (typeof plugin === "string") {
      return [plugin];
    }
  });
}

export function User2Native(config: RspackOptions): RawOptions {
  return {
    entry: config.entry ?? {},
    context: config.context,
    define: config.define,
    target: config.target,
    plugins: normalizePlugins(config.plugins ?? []),
    external: config.external,
    module: {
      // TODO: support mutliple rules to support `Module Type`
      rules: (config?.module?.rules ?? []).map(rule => {
        return {
          ...rule,
          uses: createRawModuleRuleUses(rule.uses || [])
        };
      })
    }
  };
}
