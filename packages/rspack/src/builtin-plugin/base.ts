import binding from '@rspack/binding';

import type { Compiler, RspackPluginInstance } from '..';

type AffectedHooks = keyof Compiler['hooks'];

const HOOKS_CAN_NOT_INHERENT_FROM_PARENT = [
  'make',
  'compile',
  'emit',
  'afterEmit',
  'invalid',
  'done',
  'thisCompilation',
];

export function canInherentFromParent(affectedHooks?: AffectedHooks): boolean {
  if (typeof affectedHooks === 'undefined') {
    // this arm should be removed
    return false;
  }
  return !HOOKS_CAN_NOT_INHERENT_FROM_PARENT.includes(affectedHooks);
}

export abstract class RspackBuiltinPlugin implements RspackPluginInstance {
  abstract raw(compiler: Compiler): binding.BuiltinPlugin | undefined;
  abstract name: binding.BuiltinPluginName | CustomPluginName;

  affectedHooks?: AffectedHooks;
  apply(compiler: Compiler) {
    const raw = this.raw(compiler);
    if (raw) {
      raw.canInherentFromParent = canInherentFromParent(this.affectedHooks);
      compiler.__internal__registerBuiltinPlugin(raw);
    }
  }
}

export function createBuiltinPlugin<R>(
  name: binding.BuiltinPluginName | CustomPluginName,
  options: R,
): binding.BuiltinPlugin {
  return {
    name: name as any,
    options: options ?? false, // undefined or null will cause napi error, so false for fallback
  };
}

type CustomPluginName = string;

export function create<T extends any[], R>(
  name: binding.BuiltinPluginName | CustomPluginName,
  resolve: (this: Compiler, ...args: T) => R,
  // `affectedHooks` is used to inform `createChildCompile` about which builtin plugin can be reserved.
  // However, this has a drawback as it doesn't represent the actual condition but merely serves as an indicator.
  affectedHooks?: AffectedHooks,
) {
  class Plugin extends RspackBuiltinPlugin {
    name = name;
    _args: T;
    affectedHooks = affectedHooks;

    constructor(...args: T) {
      super();
      this._args = args;
    }

    raw(compiler: Compiler): binding.BuiltinPlugin {
      return createBuiltinPlugin(name, resolve.apply(compiler, this._args));
    }
  }

  // Make the plugin class name consistent with webpack
  // https://stackoverflow.com/a/46132163
  Object.defineProperty(Plugin, 'name', { value: name });

  return Plugin;
}

const INTERNAL_PLUGIN_NAMES = Object.keys(binding.BuiltinPluginName);

/**
 * Create a wrapper class for builtin plugin.
 *
 * @example
 * ```js
 * const MyPlugin = createNativePlugin("MyPlugin", (options) => {
 * 	return options;
 * });
 *
 * new MyPlugin({
 * 	foo: "bar"
 * }).apply(compiler);
 * ```
 */
export function createNativePlugin<T extends any[], R>(
  name: CustomPluginName,
  resolve: (this: Compiler, ...args: T) => R,
  affectedHooks?: AffectedHooks,
) {
  if (INTERNAL_PLUGIN_NAMES.includes(name)) {
    throw new Error(
      `Cannot register native plugin with name '${name}', it conflicts with internal plugin names.`,
    );
  }
  return create(name, resolve, affectedHooks);
}
