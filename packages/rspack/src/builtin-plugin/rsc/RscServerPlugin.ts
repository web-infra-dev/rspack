import type binding from '@rspack/binding';
import type { Compiler } from '../..';
import { createBuiltinPlugin, RspackBuiltinPlugin } from '../base';
import { type Coordinator, GET_OR_INIT_BINDING } from './Coordinator';

/** Manifest export entry (server/client actions, module refs). */
export interface RscManifestExport {
  id: string;
  name: string;
  chunks: string[];
  cssFiles?: string[];
  async?: boolean;
}

/** Map of export name to manifest export. */
export type RscManifestNode = Record<string, RscManifestExport>;

/** Module loading config (prefix, crossOrigin). */
export interface RscModuleLoading {
  prefix: string;
  crossOrigin?: 'use-credentials' | '';
}

export interface RscManifestPerEntry {
  serverManifest: Record<string, RscManifestExport>;
  clientManifest: Record<string, RscManifestExport>;
  serverConsumerModuleMap: Record<string, RscManifestNode>;
  moduleLoading: RscModuleLoading;
  entryCssFiles: Record<string, string[]>;
  entryJsFiles: string[];
}

/** Full RSC manifest (all entries) passed to onManifest. Map from entry name to per-entry manifest. */
export type RscManifest = Record<string, RscManifestPerEntry>;

export type RscCssLinkProps = Record<string, string>;

export type RscCssLinkOptions =
  | false
  | {
      precedence?: string | false;
      attributes?: RscCssLinkProps;
    };

function toEntries(props: RscCssLinkProps): Array<[string, string]> {
  return Object.keys(props).map((key) => [key, props[key]]);
}

function normalizeCssLink(
  cssLink: RscCssLinkOptions | undefined,
): Array<[string, string]> {
  if (cssLink === false) {
    return [];
  }

  const normalizedProps: RscCssLinkProps = {
    ...(cssLink?.attributes ?? {}),
  };

  if (cssLink?.precedence === false) {
    delete normalizedProps.precedence;
  } else {
    normalizedProps.precedence = cssLink?.precedence ?? 'default';
  }

  return toEntries(normalizedProps);
}

export type RscServerPluginOptions = {
  coordinator: Coordinator;
  cssLink?: RscCssLinkOptions;
  onServerComponentChanges?: () => void | Promise<void>;
  onManifest?: (manifest: RscManifest) => void | Promise<void>;
};

export class RscServerPlugin extends RspackBuiltinPlugin {
  name = 'RscServerPlugin';
  #options: RscServerPluginOptions;

  constructor(options: RscServerPluginOptions) {
    super();
    this.#options = options;
  }

  raw(compiler: Compiler): binding.BuiltinPlugin {
    this.#options.coordinator.applyServerCompiler(compiler);

    const { coordinator, onServerComponentChanges } = this.#options;
    let onManifest: ((json: string) => void | Promise<void>) | undefined;
    if (this.#options.onManifest) {
      onManifest = (json: string) =>
        Promise.resolve(this.#options.onManifest!(JSON.parse(json)));
    }

    return createBuiltinPlugin(this.name, {
      // @ts-ignore
      coordinator: coordinator[GET_OR_INIT_BINDING](),
      cssLink: normalizeCssLink(this.#options.cssLink),
      onServerComponentChanges,
      onManifest,
    });
  }
}
