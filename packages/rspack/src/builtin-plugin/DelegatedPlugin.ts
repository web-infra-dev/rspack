import {
  BuiltinPluginName,
  type RawDelegatedPluginOptions,
} from '@rspack/binding';
import type { Compiler } from '../Compiler';
import { create } from './base';

export type DelegatedPluginOptions = {
  /**
   * The module request the delegated modules are resolved from.
   */
  source: string;

  /**
   * The way how the export of the dll bundle is used.
   */
  type: 'require' | 'object';

  /**
   * The mappings from request to module info.
   */
  content: RawDelegatedPluginOptions['content'];

  /**
   * Context of requests in the content as absolute path.
   */
  context?: string;

  /**
   * Extensions used to resolve modules in the dll bundle (only used when using 'scope').
   */
  extensions?: string[];

  /**
   * Prefix which is used for accessing the content of the dll.
   */
  scope?: string;
};

export const DelegatedPlugin = create(
  BuiltinPluginName.DelegatedPlugin,
  function (
    this: Compiler,
    options: DelegatedPluginOptions,
  ): RawDelegatedPluginOptions {
    const { source, type, content, context, extensions, scope } = options;

    return {
      source,
      type,
      content,
      context,
      extensions,
      scope,
      compilationContext: this.options.context!,
    };
  },
);
