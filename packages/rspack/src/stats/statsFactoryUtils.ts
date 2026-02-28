import type binding from '@rspack/binding';

import type { JsOriginRecord } from '@rspack/binding';
import type { Compilation } from '../Compilation';
import type { StatsOptions } from '../config';
import { compareIds, compareSelect } from '../util/comparators';
import type { StatsFactory, StatsFactoryContext } from './StatsFactory';

export type KnownStatsChunkGroup = {
  name?: string;
  chunks?: (string | number)[];
  assets?: { name: string; size?: number }[];
  filteredAssets?: number;
  assetsSize?: number;
  auxiliaryAssets?: { name: string; size?: number }[];
  filteredAuxiliaryAssets?: number;
  auxiliaryAssetsSize?: number;
  children?: {
    preload?: StatsChunkGroup[];
    prefetch?: StatsChunkGroup[];
  };
  childAssets?: {
    preload?: string[];
    prefetch?: string[];
  };
  isOverSizeLimit?: boolean;
};

export type KnownStatsChunk = {
  type: string;
  rendered: boolean;
  initial: boolean;
  entry: boolean;
  // recorded: boolean;
  reason?: string;
  size: number;
  sizes?: Record<string, number>;
  names?: string[];
  idHints?: string[];
  runtime?: string[];
  files?: string[];
  auxiliaryFiles?: string[];
  hash?: string;
  childrenByOrder?: Record<string, (string | number)[]>;
  id?: string | number;
  siblings?: (string | number)[];
  parents?: (string | number)[];
  children?: (string | number)[];
  modules?: StatsModule[];
  filteredModules?: number;
  origins?: StatsChunkOrigin[];
};

export type KnownAssetInfo = {
  immutable?: boolean;
  minimized?: boolean;
  fullhash?: string | string[];
  chunkhash?: string | string[];
  // modulehash?: string | string[];
  contenthash?: string | string[];
  sourceFilename?: string;
  copied?: boolean;
  size?: number;
  development?: boolean;
  hotModuleReplacement?: boolean;
  javascriptModule?: boolean;
  related?: Record<string, string | string[]>;
};

export type AssetInfo = KnownAssetInfo & Record<string, any>;

export type StatsChunkGroup = KnownStatsChunkGroup & Record<string, any>;

export type KnownStatsAsset = {
  type: string;
  name: string;
  info: AssetInfo;
  size: number;
  emitted: boolean;
  // comparedForEmit: boolean;
  cached: boolean;
  related?: StatsAsset[];
  chunkNames?: (string | number)[];
  chunkIdHints?: (string | number)[];
  chunks?: (string | null | undefined)[];
  auxiliaryChunkNames?: (string | number)[];
  auxiliaryChunks?: (string | null | undefined)[];
  auxiliaryChunkIdHints?: (string | number)[];
  filteredRelated?: number;
  isOverSizeLimit?: boolean;
};

export type StatsAsset = KnownStatsAsset & Record<string, any>;

export type StatsChunk = KnownStatsChunk & Record<string, any>;

export type KnownStatsModule = {
  type: string;
  moduleType: string;
  layer?: string;
  identifier?: string;
  name?: string;
  nameForCondition?: string;
  index?: number; // =preOrderIndex
  index2?: number; // =postOrderIndex
  preOrderIndex?: number;
  postOrderIndex?: number;
  size: number;
  sizes: Record<string, number>;
  cacheable?: boolean;
  built: boolean;
  codeGenerated: boolean;
  buildTimeExecuted: boolean;
  cached: boolean;
  optional?: boolean;
  orphan?: boolean;
  id?: string | number | null;
  issuerId?: string | number | null;
  chunks?: string[];
  assets?: string[];
  dependent?: boolean;
  issuer?: string;
  issuerName?: string;
  issuerPath?: StatsModuleIssuer[];
  failed?: boolean;
  errors?: number;
  warnings?: number;
  reasons?: StatsModuleReason[];
  usedExports?: boolean | string[] | null;
  providedExports?: string[] | null;
  optimizationBailout?: string[] | null;
  depth?: number;
  modules?: StatsModule[];
  filteredModules?: number;
  source?: string | Buffer;
};

export type KnownStatsProfile = {
  total: number;
  resolving: number;
  building: number;
};

export type StatsModule = KnownStatsModule & Record<string, any>;

export type KnownStatsModuleIssuer = {
  identifier?: string;
  name?: string;
  id?: string | number | null;
};

export type StatsModuleIssuer = KnownStatsModuleIssuer & Record<string, any>;

export enum StatsErrorCode {
  /**
   * Warning generated when either builtin `SwcJsMinimizer` or `LightningcssMinimizer` fails to minify the code.
   */
  ChunkMinificationError = 'ChunkMinificationError',
  /**
   * Warning generated when either builtin `SwcJsMinimizer` or `LightningcssMinimizer` fails to minify the code.
   */
  ChunkMinificationWarning = 'ChunkMinificationWarning',
  /**
   * Error generated when a module is failed to be parsed
   */
  ModuleParseError = 'ModuleParseError',
  /**
   * Warning generated when a module is failed to be parsed
   */
  ModuleParseWarning = 'ModuleParseWarning',
  /**
   * Error generated when a module is failed to be built (i.e being built by a
   * loader)
   */
  ModuleBuildError = 'ModuleBuildError',
}

export type KnownStatsError = {
  message: string;
  code?: StatsErrorCode | string;
  chunkName?: string;
  chunkEntry?: boolean;
  chunkInitial?: boolean;
  /**
   * A custom filename associated with this error/warning.
   */
  file?: string;
  /**
   * The identifier of the module related to this error/warning.
   * Usually an absolute path, may include inline loader requests.
   * @example
   * - `/path/to/project/src/index.js`
   * - `!builtin:react-refresh-loader!/path/to/project/src/index.css`
   */
  moduleIdentifier?: string;
  /**
   * The readable name of the module related to this error/warning.
   * Usually a relative path, no inline loader requests.
   * @example
   * - `"./src/index.js"`
   * - `"./src/index.css"`
   */
  moduleName?: string;
  loc?: string;
  chunkId?: string | number;
  moduleId?: string | number | null;
  moduleTrace?: StatsModuleTraceItem[];
  details?: any;
  stack?: string;
};

export type StatsError = KnownStatsError & Record<string, any>;

export type StatsModuleTraceItem = {
  originIdentifier?: string;
  originName?: string;
  moduleIdentifier?: string;
  moduleName?: string;
  originId?: string | number | null;
  moduleId?: string | number | null;
  dependencies?: StatsModuleTraceDependency[];
};

export type StatsModuleTraceDependency = KnownStatsModuleTraceDependency &
  Record<string, any>;

export type KnownStatsModuleTraceDependency = {
  loc: string;
};

export type KnownStatsModuleReason = {
  moduleIdentifier?: string;
  module?: string;
  moduleName?: string;
  resolvedModuleIdentifier?: string;
  resolvedModule?: string;
  type?: string;
  active: boolean;
  explanation?: string;
  userRequest?: string;
  loc?: string;
  moduleId?: string | number | null;
  resolvedModuleId?: string | number | null;
};

export type StatsModuleReason = KnownStatsModuleReason & Record<string, any>;

export type KnownStatsChunkOrigin = {
  module: string;
  moduleIdentifier: string;
  moduleName: string;
  loc: string;
  request: string;
  moduleId?: string | number | null;
};

export type StatsChunkOrigin = KnownStatsChunkOrigin & Record<string, any>;

export type KnownStatsCompilation = {
  /**
   * webpack version.
   * this is a hack to be compatible with plugin which detect webpack's version
   */
  version?: string;
  /** rspack version */
  rspackVersion?: string;
  name?: string;
  hash?: string;
  time?: number;
  builtAt?: number;
  publicPath?: string;
  outputPath?: string;
  assets?: StatsAsset[];
  assetsByChunkName?: Record<string, string[]>;
  chunks?: StatsChunk[];
  modules?: StatsModule[];
  entrypoints?: Record<string, StatsChunkGroup>;
  namedChunkGroups?: Record<string, StatsChunkGroup>;
  errors?: StatsError[];
  errorsCount?: number;
  warnings?: StatsError[];
  warningsCount?: number;
  filteredModules?: number;
  children?: StatsCompilation[];
  logging?: Record<string, StatsLogging>;

  // TODO: not aligned with webpack
  // env?: any;
  // needAdditionalPass?: boolean;
  // filteredAssets?: number;
};

export type StatsCompilation = KnownStatsCompilation & Record<string, any>;

export type StatsLogging = KnownStatsLogging & Record<string, any>;

export type KnownStatsLogging = {
  entries: StatsLoggingEntry[];
  filteredEntries: number;
  debug: boolean;
};

export type StatsLoggingEntry = KnownStatsLoggingEntry & Record<string, any>;

export type KnownStatsLoggingEntry = {
  type: string;
  message: string;
  trace?: string[] | undefined;
  children?: StatsLoggingEntry[] | undefined;
  args?: any[] | undefined;
  time?: number | undefined;
};

type ExtractorsByOption<T, O> = {
  [x: string]: (
    object: O,
    data: T,
    context: StatsFactoryContext,
    options: any,
    factory: StatsFactory,
  ) => void;
};

export type PreprocessedAsset = binding.JsStatsAsset & {
  type: string;
  related: PreprocessedAsset[];
  info: binding.JsStatsAssetInfo;
};

export type SimpleExtractors = {
  compilation: ExtractorsByOption<Compilation, StatsCompilation>;
  asset$visible: ExtractorsByOption<PreprocessedAsset, StatsAsset>;
  asset: ExtractorsByOption<PreprocessedAsset, StatsAsset>;
  chunkGroup: ExtractorsByOption<
    {
      name: string;
      chunkGroup: binding.JsStatsChunkGroup;
    },
    StatsChunkGroup
  >;
  module: ExtractorsByOption<binding.JsStatsModule, StatsModule>;
  module$visible: ExtractorsByOption<binding.JsStatsModule, StatsModule>;
  moduleIssuer: ExtractorsByOption<
    binding.JsStatsModuleIssuer,
    StatsModuleIssuer
  >;
  moduleReason: ExtractorsByOption<
    binding.JsStatsModuleReason,
    StatsModuleReason
  >;
  chunk: ExtractorsByOption<binding.JsStatsChunk, KnownStatsChunk>;
  chunkOrigin: ExtractorsByOption<JsOriginRecord, StatsChunkOrigin>;
  error: ExtractorsByOption<binding.JsStatsError, StatsError>;
  warning: ExtractorsByOption<binding.JsStatsError, StatsError>;
  moduleTraceItem: ExtractorsByOption<
    binding.JsStatsModuleTrace,
    StatsModuleTraceItem
  >;
  moduleTraceDependency: ExtractorsByOption<
    binding.JsStatsModuleTraceDependency,
    StatsModuleTraceDependency
  >;
};

export const iterateConfig = (
  config: Record<string, Record<string, Function>>,
  options: StatsOptions,
  fn: (a1: string, a2: Function) => void,
) => {
  for (const hookFor of Object.keys(config)) {
    const subConfig = config[hookFor];
    for (const option of Object.keys(subConfig)) {
      if (option !== '_') {
        if (option.startsWith('!')) {
          if (
            // string cannot be used as key, so use "as"
            (options as Record<string, StatsOptions[keyof StatsOptions]>)[
              option.slice(1)
            ]
          )
            continue;
        } else {
          const value = (
            options as Record<string, StatsOptions[keyof StatsOptions]>
          )[option];
          if (
            value === false ||
            value === undefined ||
            (Array.isArray(value) && value.length === 0)
          )
            continue;
        }
      }
      fn(hookFor, subConfig[option]);
    }
  }
};

type Child = {
  children?: ItemChildren;
  filteredChildren?: number;
};

type ItemChildren = Child[];

const getTotalItems = (children: ItemChildren) => {
  let count = 0;
  for (const child of children) {
    if (!child.children && !child.filteredChildren) {
      count++;
    } else {
      if (child.children) count += getTotalItems(child.children);
      if (child.filteredChildren) count += child.filteredChildren;
    }
  }
  return count;
};

export const collapse = (children: ItemChildren) => {
  // After collapse each child must take exactly one line
  const newChildren = [];
  for (const child of children) {
    if (child.children) {
      let filteredChildren = child.filteredChildren || 0;
      filteredChildren += getTotalItems(child.children);
      newChildren.push({
        ...child,
        children: undefined,
        filteredChildren,
      });
    } else {
      newChildren.push(child);
    }
  }
  return newChildren;
};

const getTotalSize = (children: ItemChildren) => {
  let size = 0;
  for (const child of children) {
    size += getItemSize(child);
  }
  return size;
};

const getItemSize = (item: Child) => {
  // Each item takes 1 line
  // + the size of the children
  // + 1 extra line when it has children and filteredChildren
  return !item.children
    ? 1
    : item.filteredChildren
      ? 2 + getTotalSize(item.children)
      : 1 + getTotalSize(item.children);
};

export const spaceLimited = (
  itemsAndGroups: ItemChildren,
  max: number,
  filteredChildrenLineReserved = false,
): {
  children: any;
  filteredChildren: any;
} => {
  if (max < 1) {
    return {
      children: undefined,
      filteredChildren: getTotalItems(itemsAndGroups),
    };
  }
  let children: any[] | undefined;
  let filteredChildren: number | undefined;
  // This are the groups, which take 1+ lines each
  const groups: ItemChildren = [];
  // The sizes of the groups are stored in groupSizes
  const groupSizes = [];
  // This are the items, which take 1 line each
  const items = [];
  // The total of group sizes
  let groupsSize = 0;

  for (const itemOrGroup of itemsAndGroups) {
    // is item
    if (!itemOrGroup.children && !itemOrGroup.filteredChildren) {
      items.push(itemOrGroup);
    } else {
      groups.push(itemOrGroup);
      const size = getItemSize(itemOrGroup);
      groupSizes.push(size);
      groupsSize += size;
    }
  }

  if (groupsSize + items.length <= max) {
    // The total size in the current state fits into the max
    // keep all
    children = groups.length > 0 ? groups.concat(items) : items;
  } else if (groups.length === 0) {
    // slice items to max
    // inner space marks that lines for filteredChildren already reserved
    const limit = max - (filteredChildrenLineReserved ? 0 : 1);
    filteredChildren = items.length - limit;
    items.length = limit;
    children = items;
  } else {
    // limit is the size when all groups are collapsed
    const limit =
      groups.length +
      (filteredChildrenLineReserved || items.length === 0 ? 0 : 1);
    if (limit < max) {
      // calculate how much we are over the size limit
      // this allows to approach the limit faster
      let oversize: number;
      // If each group would take 1 line the total would be below the maximum
      // collapse some groups, keep items
      while (
        (oversize =
          groupsSize +
          items.length +
          (filteredChildren && !filteredChildrenLineReserved ? 1 : 0) -
          max) > 0
      ) {
        // Find the maximum group and process only this one
        const maxGroupSize = Math.max(...groupSizes);
        if (maxGroupSize < items.length) {
          filteredChildren = items.length;
          items.length = 0;
          continue;
        }
        for (let i = 0; i < groups.length; i++) {
          if (groupSizes[i] === maxGroupSize) {
            const group = groups[i];
            // run this algorithm recursively and limit the size of the children to
            // current size - oversize / number of groups
            // So it should always end up being smaller
            const headerSize = group.filteredChildren ? 2 : 1;
            const limited = spaceLimited(
              group.children!,
              maxGroupSize -
                // we should use ceil to always feet in max
                Math.ceil(oversize / groups.length) -
                // we substitute size of group head
                headerSize,
              headerSize === 2,
            );
            groups[i] = {
              ...group,
              children: limited.children,
              filteredChildren: limited.filteredChildren
                ? (group.filteredChildren || 0) + limited.filteredChildren
                : group.filteredChildren,
            };
            const newSize = getItemSize(groups[i]);
            groupsSize -= maxGroupSize - newSize;
            groupSizes[i] = newSize;
            break;
          }
        }
      }
      children = groups.concat(items);
    } else if (limit === max) {
      // If we have only enough space to show one line per group and one line for the filtered items
      // collapse all groups and items
      children = collapse(groups);
      filteredChildren = items.length;
    } else {
      // If we have no space
      // collapse complete group
      filteredChildren = getTotalItems(itemsAndGroups);
    }
  }

  return {
    children,
    filteredChildren,
  };
};

export const countWithChildren = (
  compilation: Compilation,
  getItems: (compilation: Compilation, key: string) => any[],
): number => {
  let count = getItems(compilation, '').length;
  for (const child of compilation.children) {
    count += countWithChildren(child, (c, type) =>
      getItems(c, `.children[].compilation${type}`),
    );
  }
  return count;
};

// remove a prefixed "!" that can be specified to reverse sort order
const normalizeFieldKey = (field: string) => {
  if (field[0] === '!') {
    return field.slice(1);
  }
  return field;
};

// if a field is prefixed by a "!" reverse sort order
const sortOrderRegular = (field: string) => {
  if (field[0] === '!') {
    return false;
  }
  return true;
};

export const sortByField = (
  field: string,
): ((a1: Object, a2: Object) => number) => {
  if (!field) {
    const noSort = (_a: any, _b: any) => 0;
    return noSort;
  }

  const fieldKey = normalizeFieldKey(field);

  let sortFn = compareSelect(
    (m: Record<string, any>) => m[fieldKey],
    compareIds,
  );

  // if a field is prefixed with a "!" the sort is reversed!
  const sortIsRegular = sortOrderRegular(field);

  if (!sortIsRegular) {
    const oldSortFn = sortFn;
    sortFn = (a, b) => oldSortFn(b, a);
  }

  return sortFn;
};

export const assetGroup = (children: StatsAsset[]) => {
  let size = 0;
  for (const asset of children) {
    size += asset.size;
  }
  return {
    size,
  };
};

export const moduleGroup = (
  children: { size: number; sizes: Record<string, number> }[],
): { size: number; sizes: Record<string, number> } => {
  let size = 0;
  const sizes: Record<string, number> = {};
  for (const module of children) {
    size += module.size;
    for (const key of Object.keys(module.sizes)) {
      sizes[key] = (sizes[key] || 0) + module.sizes[key];
    }
  }
  return {
    size,
    sizes,
  };
};

export const mergeToObject = (
  items: {
    [key: string]: any;
    name: string;
  }[],
): Object => {
  const obj = Object.create(null);
  for (const item of items) {
    obj[item.name] = item;
  }

  return obj;
};

export const errorsSpaceLimit = (errors: StatsError[], max: number) => {
  let filtered = 0;
  // Can not fit into limit
  // print only messages
  if (errors.length + 1 >= max) {
    return {
      errors: errors.map((error) => {
        if (typeof error === 'string' || !error.details) return error;
        filtered++;
        return { ...error, details: '' };
      }),
      filtered,
    };
  }
  let fullLength = errors.length;
  let result = errors;

  let i = 0;
  for (; i < errors.length; i++) {
    const error = errors[i];
    if (typeof error !== 'string' && error.details) {
      const splitted = error.details.split('\n');
      const len = splitted.length;
      fullLength += len;
      if (fullLength > max) {
        result = i > 0 ? errors.slice(0, i) : [];
        const overLimit = fullLength - max + 1;
        const error = errors[i++];
        result.push({
          ...error,
          details: error.details!.split('\n').slice(0, -overLimit).join('\n'),
          filteredDetails: overLimit,
        });
        filtered = errors.length - i;
        for (; i < errors.length; i++) {
          const error = errors[i];
          if (typeof error === 'string' || !error.details) result.push(error);
          result.push({ ...error, details: '' });
        }
        break;
      }
      if (fullLength === max) {
        result = errors.slice(0, ++i);
        filtered = errors.length - i;
        for (; i < errors.length; i++) {
          const error = errors[i];
          if (typeof error === 'string' || !error.details) result.push(error);
          result.push({ ...error, details: '' });
        }
        break;
      }
    }
  }

  return {
    errors: result,
    filtered,
  };
};

export const warningFromStatsWarning = (
  warning: binding.JsStatsError,
): Error => {
  const res = new Error(warning.message);
  res.name = warning.name || 'StatsWarning';
  Object.assign(res, warning);
  return res;
};
