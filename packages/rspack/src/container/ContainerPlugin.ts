import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawContainerPluginOptions,
} from '@rspack/binding';
import {
  createBuiltinPlugin,
  RspackBuiltinPlugin,
} from '../builtin-plugin/base';
import type { Compiler } from '../Compiler';
import type { EntryRuntime, FilenameTemplate, LibraryOptions } from '../config';
import { parseOptions } from '../container/options';
import { normalizeShareScope, type ShareScope } from '../sharing/SharePlugin';
import { ShareRuntimePlugin } from '../sharing/ShareRuntimePlugin';

type ContainerPluginBaseOptions<Enhanced extends boolean> = {
  exposes: Exposes<Enhanced>;
  filename?: FilenameTemplate;
  library?: LibraryOptions;
  name: string;
  runtime?: EntryRuntime;
  shareScope?: ShareScope;
};
export interface ContainerPluginOptions extends ContainerPluginBaseOptions<false> {
  enhanced?: boolean;
}

export interface EnhancedContainerPluginOptions extends ContainerPluginBaseOptions<true> {
  enhanced: true;
}

type ContainerPluginConstructorOptions<Enhanced extends boolean = boolean> = [
  Enhanced,
] extends [true]
  ? EnhancedContainerPluginOptions
  : [Enhanced] extends [false]
    ? ContainerPluginBaseOptions<false> & { enhanced?: false }
    : | (ContainerPluginBaseOptions<false> & { enhanced?: false })
      | EnhancedContainerPluginOptions
      | (ContainerPluginBaseOptions<false> & { enhanced: boolean });
export type Exposes<Enhanced extends boolean = false> =
  (ExposesItem | ExposesObject<Enhanced>)[] | ExposesObject<Enhanced>;
export type ExposesItem = string;
export type ExposesItems = ExposesItem[];
export type ExposesObject<Enhanced extends boolean = false> = {
  [k: string]: ExposesConfig<Enhanced> | ExposesItem | ExposesItems;
};
type ExposesBaseConfig = {
  import: ExposesItem | ExposesItems;
  name?: string;
};
export type ExposesConfig<Enhanced extends boolean = false> = [
  Enhanced,
] extends [true]
  ? ExposesBaseConfig & { layer?: string }
  : [Enhanced] extends [false]
    ? ExposesBaseConfig & { layer?: never }
    : ExposesBaseConfig & { layer?: string };

export class ContainerPlugin<
  Enhanced extends boolean = boolean,
> extends RspackBuiltinPlugin {
  name = BuiltinPluginName.ContainerPlugin;
  _options;

  constructor(options: ContainerPluginConstructorOptions<Enhanced>) {
    super();
    const enhanced = options.enhanced ?? false;
    const shareScope = normalizeShareScope(
      options.shareScope || 'default',
      enhanced,
      'ContainerPlugin',
    );
    this._options = {
      name: options.name,
      shareScope,
      library: options.library || {
        type: 'global',
        name: options.name,
      },
      runtime: options.runtime,
      filename: options.filename,
      exposes: parseOptions(
        options.exposes,
        (item) => ({
          import: Array.isArray(item) ? item : [item],
          name: undefined,
          layer: undefined,
        }),
        (item) => {
          if (!enhanced && item.layer !== undefined) {
            throw new Error('[ContainerPlugin] layer requires enhanced=true');
          }
          return {
            import: Array.isArray(item.import) ? item.import : [item.import],
            name: item.name || undefined,
            layer: enhanced ? item.layer : undefined,
          };
        },
      ),
      enhanced,
    };
  }

  raw(compiler: Compiler): BuiltinPlugin {
    const { name, shareScope, library, runtime, filename, exposes, enhanced } =
      this._options;
    if (!compiler.options.output.enabledLibraryTypes!.includes(library.type)) {
      compiler.options.output.enabledLibraryTypes!.push(library.type);
    }
    new ShareRuntimePlugin(this._options.enhanced).apply(compiler);

    const rawOptions: RawContainerPluginOptions = {
      name,
      shareScope,
      library,
      runtime,
      filename,
      exposes: exposes.map(([key, r]) => ({ key, ...r })),
      enhanced,
    };
    return createBuiltinPlugin(this.name, rawOptions);
  }
}
