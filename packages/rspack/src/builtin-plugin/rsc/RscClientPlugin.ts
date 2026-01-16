import type binding from '@rspack/binding';
import type { Compiler } from '../..';
import { createBuiltinPlugin, RspackBuiltinPlugin } from '../base';
import { type Coordinator, GET_OR_INIT_BINDING } from './Coordinator';

export type RscClientPluginOptions = {
  coordinator: Coordinator;
};

export class RscClientPlugin extends RspackBuiltinPlugin {
  name = 'RscClientPlugin';
  #options: RscClientPluginOptions;

  constructor(options: RscClientPluginOptions) {
    super();
    this.#options = options;
  }
  raw(compiler: Compiler): binding.BuiltinPlugin {
    this.#options.coordinator.applyClientCompiler(compiler);

    return createBuiltinPlugin(this.name, {
      // @ts-ignore
      coordinator: this.#options.coordinator[GET_OR_INIT_BINDING](),
    });
  }
}
