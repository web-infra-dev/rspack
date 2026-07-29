import type { Compiler } from '../Compiler';
import type { EntryRuntime, ExternalsType, LibraryOptions } from '../config';
import {
  type Shared,
  SharePlugin,
  type ShareScope,
} from '../sharing/SharePlugin';
import { ShareRuntimePlugin } from '../sharing/ShareRuntimePlugin';
import { ContainerPlugin, type Exposes } from './ContainerPlugin';
import {
  ContainerReferencePlugin,
  type Remotes,
} from './ContainerReferencePlugin';

export interface ModuleFederationPluginV1BaseOptions<
  Enhanced extends boolean = boolean,
> {
  exposes?: Exposes<Enhanced>;
  filename?: string;
  library?: LibraryOptions;
  name: string;
  remoteType?: ExternalsType;
  remotes?: Remotes;
  runtime?: EntryRuntime;
  shareScope?: ShareScope;
  shared?: Shared;
  enhanced?: Enhanced;
}

export type ModuleFederationPluginV1Options<
  Enhanced extends boolean = boolean,
> = [Enhanced] extends [true]
  ? ModuleFederationPluginV1BaseOptions<true> & { enhanced: true }
  : [Enhanced] extends [false]
    ? ModuleFederationPluginV1BaseOptions<false> & { enhanced?: false }
    : | (ModuleFederationPluginV1BaseOptions<false> & { enhanced?: false })
      | (ModuleFederationPluginV1BaseOptions<true> & { enhanced: true })
      | (Omit<ModuleFederationPluginV1BaseOptions<boolean>, 'enhanced'> & {
          enhanced: boolean;
        });

export class ModuleFederationPluginV1<Enhanced extends boolean = boolean> {
  constructor(private _options: ModuleFederationPluginV1Options<Enhanced>) {}

  apply(compiler: Compiler) {
    const { _options: options } = this;
    const enhanced = options.enhanced ?? false;

    const library = options.library || { type: 'var', name: options.name };
    const remoteType =
      options.remoteType ||
      (options.library ? (options.library.type as ExternalsType) : 'script');

    if (
      library &&
      !compiler.options.output.enabledLibraryTypes!.includes(library.type)
    ) {
      compiler.options.output.enabledLibraryTypes!.push(library.type);
    }
    compiler.hooks.afterPlugins.tap('ModuleFederationPlugin', () => {
      new ShareRuntimePlugin(this._options.enhanced).apply(compiler);
      if (
        options.exposes &&
        (Array.isArray(options.exposes)
          ? options.exposes.length > 0
          : Object.keys(options.exposes).length > 0)
      ) {
        new ContainerPlugin({
          name: options.name,
          library,
          filename: options.filename,
          runtime: options.runtime,
          shareScope: options.shareScope,
          exposes: options.exposes,
          enhanced,
        }).apply(compiler);
      }
      if (
        options.remotes &&
        (Array.isArray(options.remotes)
          ? options.remotes.length > 0
          : Object.keys(options.remotes).length > 0)
      ) {
        new ContainerReferencePlugin({
          remoteType,
          shareScope: options.shareScope,
          remotes: options.remotes,
          enhanced,
        }).apply(compiler);
      }
      if (options.shared) {
        new SharePlugin({
          shared: options.shared,
          shareScope: options.shareScope,
          enhanced,
        }).apply(compiler);
      }
    });
  }
}
