import type { RawOptions } from '@rspack/binding';
import type { ModuleRule } from '../server';

export type Module = {
  rules?: ModuleRule[];
  parser?: RawOptions['module']['parser'];
}
