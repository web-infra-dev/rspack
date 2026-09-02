import binding from '@rspack/binding';
import type { Compiler } from '../Compiler';
import {
  type ComposeJsUseOptions,
  createRawModuleRuleUses,
} from '../config/adapterRuleUse';
import { type LoaderItem, NormalModule } from '../NormalModule';
import { parseResourceWithoutFragment } from '../util/identifier';
import type { CreatePartialRegisters } from './types';

/**
 * Snapshot of a loader as it came from the Rust side, so that entries a
 * `beforeLoaders` tap left alone can be handed back as their original index
 * instead of being resolved a second time.
 */
const ORIGINAL = Symbol('rspack.beforeLoaders.original');

type TrackedLoaderItem = LoaderItem & {
  [ORIGINAL]?: LoaderItem & { index: number };
};

function toLoaderItem(
  item: binding.JsLoaderItem,
  index: number,
  compiler: Compiler,
): TrackedLoaderItem {
  const { path, query } = parseResourceWithoutFragment(item.loader);
  let options: LoaderItem['options'] = query ? query.slice(1) : undefined;
  let ident: string | null = null;
  // `??ident` references an options object kept on the compiler, `?query` is
  // the raw options string. Mirrors `LoaderObject` in the loader runner.
  if (typeof options === 'string' && options[0] === '?') {
    ident = options.slice(1);
    options = compiler.__internal__ruleSet.references.get(
      ident,
    ) as LoaderItem['options'];
  }
  const loaderItem: TrackedLoaderItem = {
    loader: path,
    options,
    ident,
    type: item.type === '' ? null : item.type,
  };
  Object.defineProperty(loaderItem, ORIGINAL, {
    value: { ...loaderItem, index },
  });
  return loaderItem;
}

function isUntouched(item: TrackedLoaderItem, position: number) {
  const original = item[ORIGINAL];
  return (
    original !== undefined &&
    original.index === position &&
    original.loader === item.loader &&
    original.options === item.options &&
    original.ident === item.ident &&
    original.type === item.type
  );
}

function toBinding(
  item: TrackedLoaderItem,
  position: number,
  identPath: string,
  composeOptions: ComposeJsUseOptions,
): number | binding.JsAddedLoaderItem {
  const original = item[ORIGINAL];
  if (original !== undefined && isUntouched(item, original.index)) {
    return original.index;
  }
  // A loader a plugin added or rewrote still has to go through the same
  // stringify + resolve path as the loaders coming from `module.rules`.
  //
  // An ident inherited from the snapshot is dropped on the way back: it is the
  // key the rule's own options object is registered under, and reusing it for
  // rewritten options would overwrite that rule for every other module.
  const ident =
    original !== undefined && original.ident === item.ident
      ? undefined
      : (item.ident ?? undefined);
  const [use] = createRawModuleRuleUses(
    { loader: item.loader, options: item.options ?? undefined, ident },
    `${identPath}[${position}]`,
    composeOptions,
  );
  return {
    loader: use.loader,
    options: use.options,
    cache: use.cache,
    optionsCacheKey: use.optionsCacheKey,
  };
}

export const createNormalModuleHooksRegisters: CreatePartialRegisters<
  'NormalModuleBefore'
> = (getCompiler, createTap) => ({
  registerNormalModuleBeforeLoadersTaps: createTap(
    binding.RegisterJsTapKind.NormalModuleBeforeLoaders,

    () =>
      NormalModule.getCompilationHooks(
        getCompiler().__internal__get_compilation()!,
      ).beforeLoaders,

    (queried) =>
      function ({ loaders, module }: binding.JsBeforeLoadersArgs) {
        const compiler = getCompiler();
        const normalModule = module as binding.NormalModule;
        const items = loaders.map((loader, index) =>
          toLoaderItem(loader, index, compiler),
        );

        queried.call(items, normalModule);

        if (
          items.length === loaders.length &&
          items.every((item, position) => isUntouched(item, position))
        ) {
          return undefined;
        }

        const composeOptions: ComposeJsUseOptions = {
          compiler,
          mode: compiler.options.mode,
          context: compiler.options.context!,
          experiments: compiler.options
            .experiments as ComposeJsUseOptions['experiments'],
        };
        const identPath = `${normalModule.identifier()}[beforeLoaders]`;
        return items.map((item, position) =>
          toBinding(item, position, identPath, composeOptions),
        );
      },
  ),
});
