import type { RawOptions } from "@rspack/binding"

export type Entry = Record<string, string>;

export interface RspackOptions {
  /**
   * Entry points of compilation.
   */
  entry?: RawOptions['entry'];
  /**
   * An **absolute** path pointed the 
   */
  context?: RawOptions['context'],
}

export function User2Native(config: RspackOptions): RawOptions {
  return {
    entry: config.entry ?? {},
    context: config.context,
  }
}
