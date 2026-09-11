/**
 * The following code is from
 * https://github.com/webpack/loader-runner
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/loader-runner/blob/main/LICENSE
 */

import type { LoaderDefinitionFunction } from '../config';
import type { PitchLoaderDefinitionFunction } from '../config/adapterRuleUse';
import type { Compiler } from '../exports';
import type { LoaderObject } from '.';
import LoaderLoadingError from './LoaderLoadingError';
import { loadModule } from './loadModule';

type ModuleObject = {
  default?: LoaderDefinitionFunction;
  pitch?: PitchLoaderDefinitionFunction;
  raw?: boolean;
};
type LoaderModule = ModuleObject | Function;

export default function loadLoader(
  loader: LoaderObject,
  compiler: Compiler,
  callback: (err: unknown) => void,
): void {
  loadModule(loader.path, loader.type, compiler, (error, module) => {
    if (error) return callback(error);
    handleResult(loader, module as LoaderModule, callback);
  });
}

function handleResult(
  loader: LoaderObject,
  module: LoaderModule,
  callback: (err?: unknown) => void,
): void {
  if (!module || (typeof module !== 'function' && typeof module !== 'object')) {
    return callback(
      new LoaderLoadingError(
        `Module '${loader.path}' is not a loader (export function or es6 module)`,
      ),
    );
  }
  loader.normal = typeof module === 'function' ? module : module.default;
  loader.pitch = (module as ModuleObject).pitch;
  loader.raw = (module as ModuleObject).raw;
  if (!loader.pitch) {
    loader.noPitch = true;
  }
  if (
    typeof loader.normal !== 'function' &&
    typeof loader.pitch !== 'function'
  ) {
    return callback(
      new LoaderLoadingError(
        `Module '${loader.path}' is not a loader (must have normal or pitch function)`,
      ),
    );
  }
  callback();
}
