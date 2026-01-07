import {
  BuiltinPluginName,
  type RawLibManifestPluginOptions,
} from '@rspack/binding';
import { create } from './base';

export type LibManifestPluginOptions = {
  context?: string;

  entryOnly?: boolean;

  format?: boolean;

  name?: string;

  path: string;

  type?: string;
};

export const LibManifestPlugin = create(
  BuiltinPluginName.LibManifestPlugin,
  (options: LibManifestPluginOptions): RawLibManifestPluginOptions => {
    const { context, entryOnly, format, name, path, type } = options;

    return {
      context,
      entryOnly,
      format,
      name,
      path,
      type,
    };
  },
);
