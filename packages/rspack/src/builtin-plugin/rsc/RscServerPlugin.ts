import type binding from '@rspack/binding';
import type { Compiler } from '../..';
import { createBuiltinPlugin, RspackBuiltinPlugin } from '../base';
import { type Coordinator, GET_OR_INIT_BINDING } from './Coordinator';

export type RscServerPluginOptions = {
  coordinator: Coordinator;
  onServerComponentChanges?: () => Promise<void>;
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

    return createBuiltinPlugin(this.name, {
      // @ts-ignore
      coordinator: this.#options.coordinator[GET_OR_INIT_BINDING](),
      onServerComponentChanges: this.#options.onServerComponentChanges,
    });
  }
}
