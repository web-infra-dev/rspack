import path from 'node:path';
import {
  type RawEnvironment,
  type RawFuncUseCtx,
  type RawModuleRule,
  type RawModuleRuleUse,
  type RawOptions,
  type RawOutputOptions,
  type RawRuleSetCondition,
  RawRuleSetConditionType,
  type RawRuleSetLogicalConditions,
  type RawSnapshotOptions,
} from '@rspack/binding';

import type { Compiler } from '../Compiler';
import { normalizeStatsPreset } from '../Stats';
import { isNil } from '../util';
import { parseResource } from '../util/identifier';
import { isStatsColorSupported } from '../util/supportsColor';
import {
  type ComposeJsUseOptions,
  createRawModuleRuleUses,
  type LoaderContext,
  type LoaderDefinition,
  type LoaderDefinitionFunction,
  type PitchLoaderDefinitionFunction,
} from './adapterRuleUse';
import type {
  CacheNormalized,
  ExperimentsNormalized,
  ModuleOptionsNormalized,
  OutputNormalized,
  RspackOptionsNormalized,
} from './normalization';
import type {
  NewCache,
  Node,
  Optimization,
  Output,
  ParserOptionsByModuleType,
  Resolve,
  RuleSetCondition,
  RuleSetLogicalConditions,
  RuleSetRule,
  StatsValue,
} from './types';

export type {
  LoaderContext,
  LoaderDefinition,
  LoaderDefinitionFunction,
  PitchLoaderDefinitionFunction,
};

const MAX_U32 = 0xffffffff;

type ExperimentsWithDefaults = Omit<
  Required<ExperimentsNormalized>,
  'newCache'
> & {
  newCache: false | Required<NewCache>;
};

// invariant: `options` is normalized with default value applied
export const getRawOptions = (
  options: RspackOptionsNormalized,
  compiler: Compiler,
): RawOptions => {
  const mode = options.mode;
  const experiments = options.experiments as ExperimentsWithDefaults;
  const cache = options.cache!;
  return {
    name: options.name,
    mode,
    context: options.context!,
    output: getRawOutput(options.output),
    resolve: getRawResolve(options.resolve),
    resolveLoader: getRawResolve(options.resolveLoader),
    module: getRawModule(options.module, {
      compiler,
      mode,
      context: options.context!,
      experiments,
    }),
    optimization: options.optimization as Required<Optimization>,
    stats: getRawStats(options.stats),
    cache: getRawCache(cache),
    snapshot: getRawSnapshot(options.snapshot),
    experiments,
    incremental: mode === 'development' && options.incremental,
    node: getRawNode(options.node),
    amd: options.amd ? JSON.stringify(options.amd || {}) : undefined,
    bail: options.bail!,
    __references: {},
  };
};

function getRawSnapshot(
  snapshot: RspackOptionsNormalized['snapshot'],
): RawSnapshotOptions {
  const strategy = (
    value: { hash?: boolean; timestamp?: boolean } | undefined,
  ) => ({
    hash: value?.hash ?? false,
    timestamp: value?.timestamp ?? false,
  });
  return {
    immutablePaths: snapshot.immutablePaths!,
    unmanagedPaths: snapshot.unmanagedPaths!,
    managedPaths: snapshot.managedPaths!,
    buildDependencies: strategy(snapshot.buildDependencies),
    resolveBuildDependencies: strategy(snapshot.resolveBuildDependencies),
    module: strategy(snapshot.module),
    contextModule: strategy(snapshot.contextModule),
    resolve: strategy(snapshot.resolve),
  };
}

function getRawCache(cache: CacheNormalized): RawOptions['cache'] {
  if (cache === false) return false;
  if (cache.type === 'memory') {
    return {
      type: cache.type,
    };
  }
  const toRawStorageLimit = (name: string, value: number) => {
    if (value === Infinity) return 0;
    if (!Number.isSafeInteger(value) || value < 1 || value > MAX_U32) {
      throw new Error(
        `Invalid Rspack configuration: "${name}" must be a positive integer (1..${MAX_U32}) or Infinity, get \`${value}\`.`,
      );
    }
    return value;
  };
  const toRawMemoryGenerations = (value: number) => {
    if (value === Infinity) return undefined;
    if (!Number.isSafeInteger(value) || value < 0 || value > MAX_U32) {
      throw new Error(
        `Invalid Rspack configuration: "cache.maxMemoryGenerations" must be a non-negative integer (0..${MAX_U32}) or Infinity, get \`${value}\`.`,
      );
    }
    return value;
  };
  return {
    type: cache.type,
    buildDependencies: cache.buildDependencies,
    version: cache.version,
    maxAge: toRawStorageLimit('cache.maxAge', cache.maxAge!),
    maxMemoryGenerations: toRawMemoryGenerations(cache.maxMemoryGenerations!),
    storage: {
      type: cache.storage.type,
      directory: cache.storage.directory!,
      location: cache.storage.location!,
    },
    portable: cache.portable,
    readonly: cache.readonly,
  };
}

function getRawOutput(output: Output): RawOutputOptions {
  return {
    ...(output as Required<OutputNormalized>),
    environment: getRawOutputEnvironment(output.environment),
  };
}

function getRawOutputEnvironment(
  environment: Output['environment'] = {},
): RawEnvironment {
  return {
    const: Boolean(environment.const),
    computedProperty: Boolean(environment.computedProperty),
    methodShorthand: Boolean(environment.methodShorthand),
    arrowFunction: Boolean(environment.arrowFunction),
    nodePrefixForCoreModules: Boolean(environment.nodePrefixForCoreModules),
    asyncFunction: Boolean(environment.asyncFunction),
    bigIntLiteral: Boolean(environment.bigIntLiteral),
    destructuring: Boolean(environment.destructuring),
    document: Boolean(environment.document),
    dynamicImport: Boolean(environment.dynamicImport),
    dynamicImportInWorker: Boolean(environment.dynamicImportInWorker),
    forOf: Boolean(environment.forOf),
    globalThis: Boolean(environment.globalThis),
    module: Boolean(environment.module),
    optionalChaining: Boolean(environment.optionalChaining),
    logicalAssignment: Boolean(environment.logicalAssignment),
    templateLiteral: Boolean(environment.templateLiteral),
    topLevelAwait: Boolean(environment.topLevelAwait),
    importMetaDirnameAndFilename: Boolean(
      environment.importMetaDirnameAndFilename,
    ),
  };
}

function getRawExtensionAlias(
  alias: Resolve['extensionAlias'] = {},
): RawOptions['resolve']['extensionAlias'] {
  if (typeof alias === 'object' && alias !== null) {
    const entries = Object.entries(alias).map(([key, value]) => {
      if (Array.isArray(value)) {
        return [key, value];
      }
      return [key, [value]];
    });
    return Object.fromEntries(entries);
  }
  return;
}

function getRawAlias(
  alias: Resolve['alias'] = {},
): RawOptions['resolve']['alias'] {
  if (typeof alias === 'object' && alias !== null && !Array.isArray(alias)) {
    return Object.entries(alias).map(([key, value]) => ({
      path: key,
      redirect: Array.isArray(value) ? value : [value],
    }));
  }
  return false;
}

function getRawResolveByDependency(
  byDependency: Resolve['byDependency'],
): RawOptions['resolve']['byDependency'] {
  if (byDependency === undefined) return byDependency;
  return Object.fromEntries(
    Object.entries(byDependency).map(([k, v]) => [k, getRawResolve(v)]),
  );
}

function getRawTsConfig(
  tsConfig: Resolve['tsConfig'],
): RawOptions['resolve']['tsconfig'] {
  if (typeof tsConfig === 'string') {
    throw new Error('should resolve string tsConfig in normalization');
  }

  if (tsConfig === undefined) return tsConfig;
  const { configFile, references } = tsConfig;
  return {
    // WASI's cwd can differ from the JavaScript host's working directory.
    configFile: path.resolve(configFile),
    referencesType:
      references === 'auto' ? 'auto' : references ? 'manual' : 'disabled',
    references: references === 'auto' ? undefined : references,
  };
}

export function getRawResolve(resolve: Resolve): RawOptions['resolve'] {
  return {
    ...resolve,
    alias: getRawAlias(resolve.alias),
    fallback: getRawAlias(resolve.fallback),
    extensionAlias: getRawExtensionAlias(resolve.extensionAlias) as Record<
      string,
      string[]
    >,
    tsconfig: getRawTsConfig(resolve.tsConfig),
    byDependency: getRawResolveByDependency(resolve.byDependency),
  };
}

function getRawModule(
  module: ModuleOptionsNormalized,
  options: ComposeJsUseOptions,
): RawOptions['module'] {
  if (isNil(module.defaultRules)) {
    throw new Error('module.defaultRules should not be nil after defaults');
  }

  // "..." in defaultRules will be flatten in `applyModuleDefaults`, and "..." in rules is empty, so it's safe to use `as RuleSetRule[]` at here
  const ruleSet = [
    { rules: module.defaultRules as RuleSetRule[] },
    { rules: module.rules as RuleSetRule[] },
  ];
  const rules = ruleSet.map((rule, index) =>
    getRawModuleRule(rule, `ruleSet[${index}]`, options),
  );
  return {
    rules,
    parser: getRawParserOptionsMap(module.parser),
    generator: module.generator,
    noParse: module.noParse,
  };
}

function tryMatch(payload: string, condition: RuleSetCondition): boolean {
  if (typeof condition === 'string') {
    return payload.startsWith(condition);
  }

  if (condition instanceof RegExp) {
    return condition.test(payload);
  }

  if (typeof condition === 'function') {
    return condition(payload);
  }

  if (Array.isArray(condition)) {
    return condition.some((c) => tryMatch(payload, c));
  }

  if (condition && typeof condition === 'object') {
    if (condition.and) {
      return condition.and.every((c) => tryMatch(payload, c));
    }

    if (condition.or) {
      return condition.or.some((c) => tryMatch(payload, c));
    }

    if (condition.not) {
      return !tryMatch(payload, condition.not);
    }
  }

  return false;
}

const getRawModuleRule = (
  rule: RuleSetRule,
  path: string,
  options: ComposeJsUseOptions,
): RawModuleRule => {
  // Rule.loader is a shortcut to Rule.use: [ { loader } ].
  // See: https://webpack.js.org/configuration/module/#ruleloader
  const normalizedUse = rule.loader
    ? [
        {
          loader: rule.loader,
          options: rule.options,
        },
      ]
    : rule.use;
  let funcUse: undefined | ((rawContext: RawFuncUseCtx) => RawModuleRuleUse[]);
  if (typeof normalizedUse === 'function') {
    const use = normalizedUse;
    funcUse = (rawContext: RawFuncUseCtx) => {
      const context = {
        ...rawContext,
        compiler: options.compiler,
      };
      const uses = use(context);

      return createRawModuleRuleUses(uses ?? [], `${path}.use`, options);
    };
  }

  const rawModuleRule: RawModuleRule = {
    as: rule.as,
    test: rule.test ? getRawRuleSetCondition(rule.test) : undefined,
    include: rule.include ? getRawRuleSetCondition(rule.include) : undefined,
    exclude: rule.exclude ? getRawRuleSetCondition(rule.exclude) : undefined,
    issuer: rule.issuer ? getRawRuleSetCondition(rule.issuer) : undefined,
    issuerLayer: rule.issuerLayer
      ? getRawRuleSetCondition(rule.issuerLayer)
      : undefined,
    dependency: rule.dependency
      ? getRawRuleSetCondition(rule.dependency)
      : undefined,
    phase: rule.phase ? getRawRuleSetCondition(rule.phase) : undefined,
    descriptionData: rule.descriptionData
      ? Object.fromEntries(
          Object.entries(rule.descriptionData).map(([k, v]) => [
            k,
            getRawRuleSetCondition(v),
          ]),
        )
      : undefined,
    with: rule.with
      ? Object.fromEntries(
          Object.entries(rule.with).map(([k, v]) => [
            k,
            getRawRuleSetCondition(v),
          ]),
        )
      : undefined,
    resource: rule.resource ? getRawRuleSetCondition(rule.resource) : undefined,
    resourceQuery: rule.resourceQuery
      ? getRawRuleSetCondition(rule.resourceQuery)
      : undefined,
    resourceFragment: rule.resourceFragment
      ? getRawRuleSetCondition(rule.resourceFragment)
      : undefined,
    scheme: rule.scheme ? getRawRuleSetCondition(rule.scheme) : undefined,
    mimetype: rule.mimetype ? getRawRuleSetCondition(rule.mimetype) : undefined,
    sideEffects: rule.sideEffects,
    use:
      typeof normalizedUse === 'function'
        ? funcUse
        : createRawModuleRuleUses(normalizedUse ?? [], `${path}.use`, options),
    type: rule.type,
    layer: rule.layer,
    parser: getRawParser(rule.parser),
    generator: rule.generator,
    resolve: rule.resolve ? getRawResolve(rule.resolve) : undefined,
    oneOf: rule.oneOf
      ? rule.oneOf
          .filter(Boolean)
          .map((rule, index) =>
            getRawModuleRule(
              rule as RuleSetRule,
              `${path}.oneOf[${index}]`,
              options,
            ),
          )
      : undefined,
    rules: rule.rules
      ? rule.rules
          .filter(Boolean)
          .map((rule, index) =>
            getRawModuleRule(
              rule as RuleSetRule,
              `${path}.rules[${index}]`,
              options,
            ),
          )
      : undefined,
    enforce: rule.enforce,
    extractSourceMap: rule.extractSourceMap,
  };

  // Function calls may contain side-effects when interoperating with single-threaded environment.
  // In order to mitigate the issue, Rspack tries to merge these calls together.
  // See: https://github.com/web-infra-dev/rspack/issues/4003#issuecomment-1689662380
  if (
    typeof rule.test === 'function' ||
    typeof rule.resource === 'function' ||
    typeof rule.resourceQuery === 'function' ||
    typeof rule.resourceFragment === 'function'
  ) {
    delete rawModuleRule.test;
    delete rawModuleRule.resource;
    delete rawModuleRule.resourceQuery;
    delete rawModuleRule.resourceFragment;

    rawModuleRule.rspackResource = {
      type: RawRuleSetConditionType.func,
      func: (resource: string | ReturnType<typeof parseResource>) => {
        const { path, query, fragment } =
          typeof resource === 'string' ? parseResource(resource) : resource;

        if (rule.test && !tryMatch(path, rule.test)) {
          return false;
        }
        if (rule.resource && !tryMatch(path, rule.resource)) {
          return false;
        }

        if (rule.resourceQuery && !tryMatch(query, rule.resourceQuery)) {
          return false;
        }

        if (
          rule.resourceFragment &&
          !tryMatch(fragment, rule.resourceFragment)
        ) {
          return false;
        }

        return true;
      },
    };
  }
  return rawModuleRule;
};

function getRawRuleSetCondition(
  condition: RuleSetCondition,
): RawRuleSetCondition {
  if (typeof condition === 'string') {
    return {
      type: RawRuleSetConditionType.string,
      string: condition,
    };
  }
  if (condition instanceof RegExp) {
    return {
      type: RawRuleSetConditionType.regexp,
      regexp: condition,
    };
  }
  if (typeof condition === 'function') {
    return {
      type: RawRuleSetConditionType.func,
      func: condition,
    };
  }
  if (Array.isArray(condition)) {
    return {
      type: RawRuleSetConditionType.array,
      array: condition.map((i) => getRawRuleSetCondition(i)),
    };
  }
  if (typeof condition === 'object' && condition !== null) {
    return {
      type: RawRuleSetConditionType.logical,
      logical: [getRawRuleSetLogicalConditions(condition)],
    };
  }
  throw new Error(
    'unreachable: condition should be one of string, RegExp, Array, Object',
  );
}

function getRawRuleSetLogicalConditions(
  logical: RuleSetLogicalConditions,
): RawRuleSetLogicalConditions {
  return {
    and: logical.and
      ? logical.and.map((i) => getRawRuleSetCondition(i))
      : undefined,
    or: logical.or
      ? logical.or.map((i) => getRawRuleSetCondition(i))
      : undefined,
    not: logical.not ? getRawRuleSetCondition(logical.not) : undefined,
  };
}

function getRawParserOptionsMap(
  parser: ParserOptionsByModuleType,
): Record<string, Record<string, unknown>> {
  const json = parser.json;
  const rawJson = getRawParser(json);
  return rawJson && rawJson !== json ? { ...parser, json: rawJson } : parser;
}

function getRawParser(
  parser: RuleSetRule['parser'],
): Record<string, unknown> | undefined {
  if (!parser || typeof parser.parse !== 'function') {
    return parser;
  }
  const parse = (source: string) => JSON.stringify(parser.parse(source));
  // Keep named property reads on the original object when adapting the callback.
  return new Proxy(
    {},
    {
      get: (_target, key) =>
        key === 'parse' ? parse : Reflect.get(parser, key),
    },
  );
}

function getRawNode(node: Node): RawOptions['node'] {
  if (node === false) {
    return undefined;
  }

  if (isNil(node.__dirname) || isNil(node.global) || isNil(node.__filename)) {
    throw new Error(
      'node.__dirname, node.global, node.__filename should not be nil',
    );
  }

  return {
    dirname: String(node.__dirname),
    filename: String(node.__filename),
    global: String(node.global),
  };
}

function getRawStats(stats: StatsValue): RawOptions['stats'] {
  const statsOptions = normalizeStatsPreset(stats);
  const colors =
    statsOptions.colors === undefined
      ? isStatsColorSupported()
      : Boolean(statsOptions.colors);
  return {
    colors,
  };
}
