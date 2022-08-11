import type { RawOptions } from "@rspack/binding"

export type Plugin = string | [string] | [string, unknown];

export interface RspackOptions {
  /**
   * Entry points of compilation.
   */
  entry?: RawOptions['entry'];
  /**
   * An **absolute** path pointed the 
   */
  context?: RawOptions['context'],
  /**
   * An array of plugins
   */
  plugins?: Plugin[],
}

export function normalizePlugins(plugins: Plugin[]) {
  return plugins.map(plugin => {
    if (typeof plugin === 'string') {
      return [plugin]
    }
  })
}

export function User2Native(config: RspackOptions): RawOptions {
  return {
    entry: config.entry ?? {},
    context: config.context,
    plugins: normalizePlugins(config.plugins),
  }
}
