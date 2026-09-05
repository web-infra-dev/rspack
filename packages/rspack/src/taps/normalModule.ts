import binding from '@rspack/binding';
import type { Compiler } from '../Compiler';
import {
  type ComposeJsUseOptions,
  createRawModuleRuleUses,
} from '../config/adapterRuleUse';
import type { RuleSetLoaderWithOptions } from '../config/types';
import { type LoaderItem, NormalModule } from '../NormalModule';
import { parseResourceWithoutFragment } from '../util/identifier';
import type { CreatePartialRegisters } from './types';

/**
 * Snapshot of a loader as it came from the Rust side, so that entries a
 * `beforeLoaders` tap left alone can be handed back as their original index
 * instead of being resolved a second time.
 */
const ORIGINAL = Symbol('rspack.beforeLoaders.original');

interface Snapshot extends LoaderItem {
  index: number;
  /** `Rule.use[].cache`, carried over when a tap rewrites the entry. */
  cache: boolean;
  /** `Rule.use[].parallel`, carried over when a tap rewrites the entry. */
  parallel: RuleSetLoaderWithOptions['parallel'];
}

type TrackedLoaderItem = LoaderItem & {
  [ORIGINAL]?: Snapshot;
};

function toLoaderItem(
  item: binding.JsBeforeLoadersLoaderItem,
  index: number,
  compiler: Compiler,
): TrackedLoaderItem {
  const { path, query } = parseResourceWithoutFragment(item.request);
  let options: LoaderItem['options'] = query ? query.slice(1) : undefined;
  let ident: string | null = null;
  let parallel: Snapshot['parallel'];
  // `??ident` references an options object kept on the compiler, `?query` is
  // the raw options string. Mirrors `LoaderObject` in the loader runner.
  if (typeof options === 'string' && options[0] === '?') {
    ident = options.slice(1);
    options = compiler.__internal__ruleSet.references.get(
      ident,
    ) as LoaderItem['options'];
    parallel = compiler.__internal__ruleSet.references.get(
      `${ident}$$parallelism`,
    ) as Snapshot['parallel'];
  }
  const loaderItem: TrackedLoaderItem = {
    loader: path,
    options,
    ident,
    type: item.type ?? null,
  };
  Object.defineProperty(loaderItem, ORIGINAL, {
    value: { ...loaderItem, index, cache: item.cache, parallel } as Snapshot,
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
  configured: Map<string, Snapshot>,
): number | binding.JsAddedLoaderItem {
  const original = item[ORIGINAL];
  if (original !== undefined && isUntouched(item, original.index)) {
    return original.index;
  }
  // An ident that came from the configuration is dropped on the way back: it is
  // the key that rule's own options object is registered under, and reusing it
  // for rewritten options would overwrite the options every other module
  // matched by that rule receives.
  //
  // This is checked against the idents seen in this call rather than against
  // the entry's own snapshot, so that an entry a tap rebuilt from scratch
  // (`{ ...loaders[0], options }`, which drops the snapshot) is caught too.
  const inherited =
    item.ident !== null ? configured.get(item.ident) : undefined;
  const ident = inherited === undefined ? (item.ident ?? undefined) : undefined;
  // `parallel` requires an options object; a tap may have replaced it with a
  // query string, in which case the loader can no longer run in parallel.
  const parallel =
    typeof item.options === 'object' && item.options !== null
      ? (original?.parallel ?? inherited?.parallel)
      : undefined;
  // A loader a plugin added or rewrote still has to go through the same
  // stringify + resolve path as the loaders coming from `module.rules`.
  const [use] = createRawModuleRuleUses(
    {
      loader: item.loader,
      options: item.options ?? undefined,
      ident,
      cache: original?.cache ?? inherited?.cache,
      parallel,
    },
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
        // Collected before the taps run: an entry a tap rebuilds from scratch
        // no longer carries its snapshot, but its ident is still the one the
        // configuration registered.
        const configured = new Map<string, Snapshot>();
        for (const item of items) {
          const original = item[ORIGINAL] as Snapshot;
          if (original.ident !== null && !configured.has(original.ident)) {
            configured.set(original.ident, original);
          }
        }

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
        // `|` separates the type from the path in a loader identifier on the
        // Rust side, so it must not end up inside a generated ident.
        const identPath = `${normalModule
          .identifier()
          .replace(/\|/g, '%7C')}[beforeLoaders]`;
        return items.map((item, position) =>
          toBinding(item, position, identPath, composeOptions, configured),
        );
      },
  ),
});
