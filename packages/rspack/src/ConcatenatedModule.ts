import binding from '@rspack/binding';
import { Module } from './Module';

export class ConcatenatedModule extends Module {
  get rootModule(): Module {
    return binding.concatenatedModuleGetRootModule(this);
  }

  get modules(): Module[] {
    return binding.concatenatedModuleGetModules(this);
  }
}

for (const name of ['rootModule', 'modules']) {
  Object.defineProperty(ConcatenatedModule.prototype, name, {
    enumerable: true,
  });
}
