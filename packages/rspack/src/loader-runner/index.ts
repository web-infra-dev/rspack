/**
 * The following code is modified based on
 * https://github.com/webpack/loader-runner
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/loader-runner/blob/main/LICENSE
 */
import querystring from 'node:querystring';
import { format } from 'node:util';
import {
  formatDiagnostic,
  type JsLoaderContext,
  type JsLoaderItem,
  JsLoaderState,
  JsRspackSeverity,
} from '@rspack/binding';
import {
  OriginalSource,
  RawSource,
  type Source,
  SourceMapSource,
} from 'webpack-sources';

import { commitCustomFieldsToRust } from '../BuildInfo';
import { Compilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import {
  BUILTIN_LOADER_PREFIX,
  type Diagnostic,
  isUseSimpleSourceMap,
  isUseSourceMap,
  type LoaderContext,
  type ResolveCallback,
} from '../config/adapterRuleUse';
import { NormalModule } from '../NormalModule';
import { Resolver, type ResolveContext } from '../Resolver';
import { NonErrorEmittedError, type RspackError } from '../RspackError';
import { JavaScriptTracer } from '../trace';
import {
  isNil,
  serializeObject,
  stringifyLoaderObject,
  toBuffer,
  toObject,
} from '../util';
import { createHash } from '../util/createHash';
import {
  absolutify,
  contextify,
  makePathsRelative,
  parseResource,
  parseResourceWithoutFragment,
} from '../util/identifier';
import { memoize } from '../util/memoize';
import { ModuleError, ModuleWarning } from './ModuleError';
import { LoaderCache, type LoaderCacheEntry } from './cache';
import { LoaderDependenciesState } from './dependencies';
import {
  deserializeLoaderOptions,
  getLoaderAdditionalData,
  getLoaderCompilerBridge,
  getLoaderInputFileSystem,
  getLoaderOptions,
  markLoaderFunctionThis,
  registerLoaderAdditionalData,
  serializeLoaderOptions,
} from './service';
import {
  convertArgs,
  extractLoaderName,
  loadLoader,
  runSyncOrAsync,
} from './utils';

const LOADER_PROCESS_NAME = 'Loader Analysis';

type LoaderObjectOptions = string | (object & { ident?: unknown }) | null;

function stringifyLoaderRequest(path: string, query: string, fragment: string) {
  return (
    path.replace(/#/g, '\u200b#') + query.replace(/#/g, '\u200b#') + fragment
  );
}

export class LoaderObject {
  request: string;
  path: string;
  query: string;
  fragment: string;
  options?: LoaderObjectOptions;
  ident: string | null;
  normal?: Function | null;
  pitch?: Function | null;
  raw?: boolean | null;
  type?: 'module' | 'commonjs';
  parallel?: boolean | { maxWorkers?: number };
  /**
   * @internal This field is rspack internal. Do not edit.
   */
  loaderItem: JsLoaderItem;

  constructor(loaderItem: JsLoaderItem, compiler: Compiler) {
    const optionsHandle = loaderItem.optionsHandle;
    const splittedRequest = parseResourceWithoutFragment(loaderItem.loader);
    this.path = splittedRequest.path;
    this.fragment = '';
    this.options = splittedRequest.query
      ? splittedRequest.query.slice(1)
      : undefined;
    this.ident = null;
    this.normal = null;
    this.pitch = null;
    this.raw = null;

    const referencedIdent =
      loaderItem.ident ??
      (typeof this.options === 'string' && this.options[0] === '?'
        ? this.options.slice(1)
        : null);
    const referencedOptions =
      optionsHandle !== null && optionsHandle !== undefined
        ? getLoaderOptions(optionsHandle)
        : undefined;
    if (referencedOptions !== undefined) {
      this.options = referencedOptions as LoaderObjectOptions;
      this.ident = referencedIdent;
    } else if (referencedIdent !== null) {
      const ident = referencedIdent;
      if (ident === '[[missing ident]]') {
        throw new Error(
          'No ident is provided by referenced loader. ' +
            'When using a function for Rule.use in config you need to ' +
            "provide an 'ident' property for referenced loader options.",
        );
      }
      this.options = compiler.__internal__ruleSet.references.get(ident) as
        LoaderObjectOptions | undefined;
      if (this.options === undefined) {
        throw new Error('Invalid ident is provided by referenced loader');
      }
      this.ident = ident;
    }

    // CHANGE: `rspack_core` returns empty string for `undefined` type.
    // Comply to webpack test case: tests/webpack-test/cases/loaders/cjs-loader-type/index.js
    this.type =
      loaderItem.type === ''
        ? undefined
        : (loaderItem.type as LoaderObject['type']);
    if (this.options === null) this.query = '';
    else if (this.options === undefined) this.query = '';
    else if (typeof this.options === 'string') this.query = `?${this.options}`;
    else if (this.ident) this.query = `??${this.ident}`;
    else if (this.options.ident) this.query = `??${this.options.ident}`;
    else this.query = `?${JSON.stringify(this.options)}`;

    this.request = stringifyLoaderRequest(this.path, this.query, this.fragment);
    this.parallel = loaderItem.parallel;
    this.loaderItem = loaderItem;
    this.loaderItem.data = this.loaderItem.data ?? {};
  }

  get pitchExecuted() {
    return this.loaderItem.pitchExecuted;
  }

  set pitchExecuted(value: boolean) {
    if (!value) {
      throw new Error('pitchExecuted should be true');
    }

    this.loaderItem.pitchExecuted = true;
  }

  get normalExecuted() {
    return this.loaderItem.normalExecuted;
  }

  set normalExecuted(value: boolean) {
    if (!value) {
      throw new Error('normalExecuted should be true');
    }

    this.loaderItem.normalExecuted = true;
  }

  set noPitch(value: boolean) {
    if (!value) {
      throw new Error('noPitch should be true');
    }
    this.loaderItem.noPitch = true;
  }

  shouldYield() {
    return this.request.startsWith(BUILTIN_LOADER_PREFIX);
  }

  static __from_binding(
    loaderItem: JsLoaderItem,
    compiler: Compiler,
  ): LoaderObject {
    return new this(loaderItem, compiler);
  }

  static __to_binding(loader: LoaderObject): JsLoaderItem {
    return loader.loaderItem;
  }
}

class JsSourceMap {
  static __from_binding(map?: Uint8Array) {
    return isNil(map) ? undefined : toObject(map);
  }

  static __to_binding(map?: string | object | null) {
    return serializeObject(map);
  }
}

function dirname(path: string) {
  if (path === '/') return '/';
  const i = path.lastIndexOf('/');
  const j = path.lastIndexOf('\\');
  const i2 = path.indexOf('/');
  const j2 = path.indexOf('\\');
  const idx = i > j ? i : j;
  const idx2 = i > j ? i2 : j2;
  if (idx < 0) return path;
  if (idx === idx2) return path.slice(0, idx + 1);
  return path.slice(0, idx);
}

function getCurrentLoader(
  loaderContext: LoaderContext,
  index = loaderContext.loaderIndex,
) {
  if (
    loaderContext.loaders?.length &&
    index < loaderContext.loaders.length &&
    index >= 0 &&
    loaderContext.loaders[index]
  ) {
    return loaderContext.loaders[index];
  }
  return null;
}

function runLoadersInternal(
  compiler: Compiler,
  context: JsLoaderContext,
  worker = false,
): Promise<JsLoaderContext> {
  const hooksOnly = context.__internal__runHooksOnly;
  const loaderState = context.loaderState;
  const pitch = loaderState === JsLoaderState.Pitching;

  const { resource } = context;
  const traceData = JavaScriptTracer.isEnabled()
    ? {
        uuid: JavaScriptTracer.uuid(),
        args: {
          is_pitch: pitch,
          resource: resource,
        },
      }
    : undefined;

  if (traceData) {
    JavaScriptTracer.startAsync({
      name: 'run_js_loaders',
      processName: LOADER_PROCESS_NAME,
      uuid: traceData.uuid,
      ph: 'b',
      args: traceData.args,
    });
  }
  const splittedResource = resource && parseResource(resource);
  const resourcePath = splittedResource ? splittedResource.path : undefined;
  const resourceQuery = splittedResource ? splittedResource.query : undefined;
  const resourceFragment = splittedResource
    ? splittedResource.fragment
    : undefined;
  const contextDirectory = resourcePath ? dirname(resourcePath) : null;

  // execution state
  const dependencies = new LoaderDependenciesState(context.dependencies);
  const loaderCache = context.__internal__loaderCache
    ? new LoaderCache(context, dependencies)
    : undefined;

  /// Construct `loaderContext`
  const loaderContext = {} as LoaderContext;
  (loaderContext as any).parallel = worker;

  loaderContext.loaders = context.loaderItems.map((item) => {
    return LoaderObject.__from_binding(item, compiler);
  });

  loaderContext.hot = context.hot;
  loaderContext.context = contextDirectory;
  loaderContext.resourcePath = resourcePath!;
  loaderContext.resourceQuery = resourceQuery!;
  loaderContext.resourceFragment = resourceFragment!;
  loaderContext.dependency = loaderContext.addDependency =
    function addDependency(file) {
      dependencies.addFile(file);
    };
  loaderContext.addContextDependency = function addContextDependency(context) {
    dependencies.addContext(context);
  };
  loaderContext.addMissingDependency = function addMissingDependency(context) {
    dependencies.addMissing(context);
  };
  loaderContext.addBuildDependency = function addBuildDependency(file) {
    dependencies.addBuild(file);
  };
  loaderContext.getDependencies = function getDependencies() {
    return dependencies.fileDependencies();
  };
  loaderContext.getContextDependencies = function getContextDependencies() {
    return dependencies.contextDependencies();
  };
  loaderContext.getMissingDependencies = function getMissingDependencies() {
    return dependencies.missingDependencies();
  };
  loaderContext.clearDependencies = function clearDependencies() {
    dependencies.clearDependencies();
    context.cacheable = true;
  };

  loaderContext.importModule = function importModule(
    request,
    userOptions,
    callback,
  ) {
    if (traceData) {
      JavaScriptTracer.startAsync({
        name: 'importModule',
        processName: LOADER_PROCESS_NAME,
        uuid: traceData.uuid,
        args: traceData.args,
      });
    }
    const options = userOptions ? userOptions : {};
    const context = loaderContext;
    function finalCallback(
      onError: (err: Error) => void,
      onDone: (res: any) => void,
    ) {
      return function (err?: Error, res?: any) {
        if (err) {
          if (traceData) {
            JavaScriptTracer.endAsync({
              name: 'importModule',
              processName: LOADER_PROCESS_NAME,
              uuid: traceData.uuid,
              args: traceData.args,
            });
          }
          onError(err);
        } else {
          for (const dep of res.buildDependencies) {
            context.addBuildDependency(dep);
          }
          for (const dep of res.contextDependencies) {
            context.addContextDependency(dep);
          }
          for (const dep of res.missingDependencies) {
            context.addMissingDependency(dep);
          }
          for (const dep of res.fileDependencies) {
            context.addDependency(dep);
          }
          if (res.cacheable === false) {
            context.cacheable(false);
          }
          if (traceData) {
            JavaScriptTracer.endAsync({
              name: 'importModule',
              processName: LOADER_PROCESS_NAME,
              uuid: traceData.uuid,
              args: traceData.args,
            });
          }
          if (res.error) {
            onError(
              compiler.__internal__takeModuleExecutionResult(res.id) ??
                new Error(res.error),
            );
          } else {
            onDone(compiler.__internal__takeModuleExecutionResult(res.id));
          }
        }
      };
    }
    if (!callback) {
      return new Promise((resolve, reject) => {
        compiler
          ._lastCompilation!.__internal_getInner()
          .importModule(
            request,
            options.layer,
            options.publicPath,
            options.baseUri,
            context._module.identifier(),
            loaderContext.context,
            finalCallback(reject, resolve),
          );
      });
    }
    return compiler._lastCompilation!.__internal_getInner().importModule(
      request,
      options.layer,
      options.publicPath,
      options.baseUri,
      context._module.identifier(),
      loaderContext.context,
      finalCallback(
        (err) => callback(err),
        (res) => callback(undefined, res),
      ),
    );
  } as LoaderContext['importModule'];
  Object.defineProperty(loaderContext, 'resource', {
    enumerable: true,
    get: () => {
      if (loaderContext.resourcePath === undefined) return undefined;
      return (
        loaderContext.resourcePath.replace(/#/g, '\u200b#') +
        loaderContext.resourceQuery.replace(/#/g, '\u200b#') +
        loaderContext.resourceFragment
      );
    },
    set: (value) => {
      const splittedResource = value && parseResource(value);
      loaderContext.resourcePath = splittedResource
        ? splittedResource.path
        : undefined;
      loaderContext.resourceQuery = splittedResource
        ? splittedResource.query
        : undefined;
      loaderContext.resourceFragment = splittedResource
        ? splittedResource.fragment
        : undefined;
    },
  });
  Object.defineProperty(loaderContext, 'request', {
    enumerable: true,
    get: () =>
      loaderContext.loaders
        .map((o) => o.request)
        .concat(loaderContext.resource || '')
        .join('!'),
  });
  Object.defineProperty(loaderContext, 'remainingRequest', {
    enumerable: true,
    get: () => {
      if (
        loaderContext.loaderIndex >= loaderContext.loaders.length - 1 &&
        !loaderContext.resource
      )
        return '';
      return loaderContext.loaders
        .slice(loaderContext.loaderIndex + 1)
        .map((o) => o.request)
        .concat(loaderContext.resource || '')
        .join('!');
    },
  });
  Object.defineProperty(loaderContext, 'currentRequest', {
    enumerable: true,
    get: () =>
      loaderContext.loaders
        .slice(loaderContext.loaderIndex)
        .map((o) => o.request)
        .concat(loaderContext.resource || '')
        .join('!'),
  });
  Object.defineProperty(loaderContext, 'previousRequest', {
    enumerable: true,
    get: () =>
      loaderContext.loaders
        .slice(0, loaderContext.loaderIndex)
        .map((o) => o.request)
        .join('!'),
  });
  Object.defineProperty(loaderContext, 'query', {
    enumerable: true,
    get: () => {
      const entry = loaderContext.loaders[loaderContext.loaderIndex];
      return entry.options && typeof entry.options === 'object'
        ? entry.options
        : entry.query;
    },
  });
  loaderContext.version = 2;
  loaderContext.sourceMap = compiler.options.devtool
    ? isUseSourceMap(compiler.options.devtool)
    : (context._module.useSourceMap ?? false);
  loaderContext.mode = compiler.options.mode;
  Object.assign(loaderContext, compiler.options.loader);
  let hookLoaderContextExtensions: Record<string, any> = {};

  const getResolveContext = () => {
    return {
      fileDependencies: {
        add: (d) => {
          loaderContext.addDependency(d);
        },
      },
      contextDependencies: {
        add: (d) => {
          loaderContext.addContextDependency(d);
        },
      },
      missingDependencies: {
        add: (d) => {
          loaderContext.addMissingDependency(d);
        },
      },
    } as ResolveContext;
  };

  const getResolver = memoize(() => {
    return compiler._lastCompilation!.resolverFactory.get('normal');
  });

  loaderContext.resolve = function resolve(context, request, callback) {
    getResolver().resolve({}, context, request, getResolveContext(), callback);
  };

  loaderContext.getResolve = function getResolve(options) {
    const resolver = getResolver();
    const child = options ? resolver.withOptions(options) : resolver;

    function resolveWithOptions(
      context: string,
      request: string,
      callback: ResolveCallback,
    ): void;
    function resolveWithOptions(
      context: string,
      request: string,
    ): Promise<string | false | undefined>;
    function resolveWithOptions(
      context: string,
      request: string,
      callback?: ResolveCallback,
    ) {
      if (callback) {
        child.resolve({}, context, request, getResolveContext(), callback);
        return;
      }
      // TODO: (type) our native resolver return value is "string | false" but webpack type is "string"
      return new Promise<string | false | undefined>((resolve, reject) => {
        child.resolve(
          {},
          context,
          request,
          getResolveContext(),
          (err, result) => {
            if (err) reject(err);
            else resolve(result);
          },
        );
      });
    }

    return resolveWithOptions;
  };
  loaderContext.getLogger = function getLogger(name) {
    return compiler._lastCompilation!.getLogger(
      [name, resource].filter(Boolean).join('|'),
    );
  };
  loaderContext.rootContext = compiler.context;
  // The public API intentionally accepts only Error instances. Keep these runtime checks for
  // untyped JavaScript loaders that pass strings or other non-Error values.
  loaderContext.emitError = function emitError(e) {
    if (!(e instanceof Error)) {
      e = new NonErrorEmittedError(e);
    }
    const error = new ModuleError(e, {
      from: stringifyLoaderObject(
        loaderContext.loaders[loaderContext.loaderIndex],
      ),
    });
    error.module = loaderContext._module;
    compiler._lastCompilation!.__internal__pushRspackDiagnostic({
      error,
      severity: JsRspackSeverity.Error,
    });
  };
  loaderContext.emitWarning = function emitWarning(e) {
    if (!(e instanceof Error)) {
      e = new NonErrorEmittedError(e);
    }
    const warning = new ModuleWarning(e, {
      from: stringifyLoaderObject(
        loaderContext.loaders[loaderContext.loaderIndex],
      ),
    });
    warning.module = loaderContext._module;
    compiler._lastCompilation!.__internal__pushRspackDiagnostic({
      error: warning,
      severity: JsRspackSeverity.Warn,
    });
  };
  loaderContext.emitFile = function emitFile(
    name,
    content,
    sourceMap?,
    assetInfo?,
  ) {
    let source: Source | undefined;
    if (sourceMap) {
      if (
        typeof sourceMap === 'string' &&
        (loaderContext.sourceMap ||
          (compiler.options.devtool &&
            isUseSimpleSourceMap(compiler.options.devtool)))
      ) {
        source = new OriginalSource(
          content,
          makePathsRelative(contextDirectory!, sourceMap, compiler),
        );
      }

      if (loaderContext.sourceMap) {
        source = new SourceMapSource(
          content,
          name,
          makePathsRelative(contextDirectory!, sourceMap, compiler),
        );
      }
    } else {
      source = new RawSource(content);
    }
    loaderContext._module.emitFile(name, source!, assetInfo);
  };
  loaderContext.fs = compiler.inputFileSystem;
  loaderContext.experiments = {
    emitDiagnostic: (diagnostic: Diagnostic) => {
      const d = Object.assign({}, diagnostic, {
        message:
          diagnostic.severity === 'warning'
            ? `ModuleWarning: ${diagnostic.message}`
            : `ModuleError: ${diagnostic.message}`,
        moduleIdentifier: context._module.identifier(),
      });
      compiler._lastCompilation!.__internal__pushDiagnostic(
        formatDiagnostic(d),
      );
    },
  };

  const getAbsolutify = memoize(() => absolutify.bindCache(compiler.root));
  const getAbsolutifyInContext = memoize(() =>
    absolutify.bindContextCache(contextDirectory!, compiler.root),
  );
  const getContextify = memoize(() => contextify.bindCache(compiler.root));
  const getContextifyInContext = memoize(() =>
    contextify.bindContextCache(contextDirectory!, compiler.root),
  );

  loaderContext.utils = {
    absolutify: (context, request) => {
      return context === contextDirectory
        ? getAbsolutifyInContext()(request)
        : getAbsolutify()(context, request);
    },
    contextify: (context, request) => {
      return context === contextDirectory
        ? getContextifyInContext()(request)
        : getContextify()(context, request);
    },
    createHash: (type) => {
      return createHash(
        type || compiler._lastCompilation!.outputOptions.hashFunction!,
      );
    },
  };

  loaderContext._compiler = compiler;
  loaderContext._compilation = compiler._lastCompilation!;
  loaderContext._module = context._module;

  loaderContext.getOptions = () => {
    const loader = getCurrentLoader(loaderContext);
    let options = loader?.options;

    if (typeof options === 'string') {
      if (options.startsWith('{') && options.endsWith('}')) {
        try {
          options = JSON.parse(options);
        } catch (e: any) {
          throw new Error(
            `JSON parsing failed for loader's string options: ${e.message}`,
          );
        }
      } else {
        options = querystring.parse(options);
      }
    }

    if (options === null || options === undefined) {
      options = {};
    }

    return options;
  };

  const contextBeforeHooks = hooksOnly
    ? new Map(
        Reflect.ownKeys(loaderContext).map((key) => [
          key,
          Object.getOwnPropertyDescriptor(loaderContext, key),
        ]),
      )
    : undefined;
  let compilation: Compilation | undefined =
    worker && !hooksOnly ? undefined : compiler._lastCompilation;
  let step = 0;
  while (compilation) {
    NormalModule.getCompilationHooks(compilation).loader.call(
      loaderContext,
      loaderContext._module,
    );
    compilation = compilation.compiler.parentCompilation;
    step++;
    if (step > 1000) {
      throw Error(
        'Too many nested child compiler, exceeded max limitation 1000',
      );
    }
  }
  dependencies.mergeChanges();
  if (hooksOnly) {
    const hookExtensions: Record<string, any> = {
      ...(compiler.options.loader ?? {}),
    };
    for (const key of Reflect.ownKeys(loaderContext)) {
      const before = contextBeforeHooks!.get(key);
      const after = Object.getOwnPropertyDescriptor(loaderContext, key);
      if (
        typeof key === 'string' &&
        after &&
        'value' in after &&
        (!before ||
          !('value' in before) ||
          !Object.is(before.value, after.value))
      ) {
        hookExtensions[key] = after.value;
      }
    }
    context.__internal__hookExtensions = serializeLoaderOptions(hookExtensions);
    context.loaderItems = loaderContext.loaders.map((item) =>
      LoaderObject.__to_binding(item),
    );
    return Promise.resolve(context);
  }
  if (worker && context.__internal__hookExtensions) {
    hookLoaderContextExtensions = deserializeLoaderOptions(
      context.__internal__hookExtensions,
    );
    Object.assign(loaderContext, hookLoaderContextExtensions);
  }

  markLoaderFunctionThis(loaderContext, {
    ...(compiler.options.loader ?? {}),
    ...hookLoaderContextExtensions,
  });

  /// Sync with `context`
  Object.defineProperty(loaderContext, 'loaderIndex', {
    enumerable: true,
    get: () => context.loaderIndex,
    set: (loaderIndex) => (context.loaderIndex = loaderIndex),
  });
  Object.defineProperty(loaderContext, 'cacheable', {
    enumerable: true,
    get: () => (cacheable?: boolean) => {
      if (cacheable === false) {
        context.cacheable = cacheable;
      }
    },
  });
  Object.defineProperty(loaderContext, 'data', {
    enumerable: true,
    get: () => loaderContext.loaders[loaderContext.loaderIndex].loaderItem.data,
    set: (data) =>
      (loaderContext.loaders[loaderContext.loaderIndex].loaderItem.data = data),
  });

  /// Rspack private
  loaderContext.__internal__setParseMeta = (key: string, value: string) => {
    context.__internal__parseMeta[key] = value;
  };

  const enableParallelism = (currentLoaderObject: any) => {
    // A buffer backed by WASM linear memory retains the entire backing store
    // after crossing the worker boundary, so a cached loader must stay on the
    // main thread to avoid copying the whole WASM memory through N-API.
    if (process.env.WASM && currentLoaderObject?.loaderItem.cache) return false;

    return currentLoaderObject?.parallel;
  };

  const isomorphoicRun = async (fn: Function, args: any[]) => {
    const currentLoaderObject = getCurrentLoader(loaderContext);
    const parallelism = enableParallelism(currentLoaderObject);
    let loaderName: string | undefined;

    if (traceData || parallelism) {
      loaderName = extractLoaderName(currentLoaderObject!.request);
    }

    if (traceData) {
      JavaScriptTracer.startAsync({
        name: loaderName!,
        trackName: loaderName!,
        processName: LOADER_PROCESS_NAME,
        uuid: traceData.uuid,
        args: traceData.args,
      });
    }

    if (loaderState === JsLoaderState.Normal)
      convertArgs(args, !!currentLoaderObject?.raw);
    const result = (await runSyncOrAsync(fn, loaderContext, args)) || [];

    if (traceData) {
      JavaScriptTracer.endAsync({
        name: loaderName!,
        trackName: loaderName!,
        processName: LOADER_PROCESS_NAME,
        uuid: traceData.uuid,
        args: traceData.args,
      });
    }

    return result;
  };

  const executeLoaders = async (): Promise<JsLoaderContext> => {
    try {
      switch (loaderState) {
        case JsLoaderState.Pitching: {
          while (loaderContext.loaderIndex < loaderContext.loaders.length) {
            const currentLoaderObject =
              loaderContext.loaders[loaderContext.loaderIndex];
            const parallelism = enableParallelism(currentLoaderObject);

            if (currentLoaderObject.shouldYield()) break;
            if (!!parallelism !== worker) break;
            if (currentLoaderObject.pitchExecuted) {
              loaderContext.loaderIndex += 1;
              continue;
            }

            await loadLoader(currentLoaderObject, compiler);
            const fn = currentLoaderObject.pitch;
            // If parallelism is enabled,
            // we delegate the current loader to use the runner in worker.
            currentLoaderObject.pitchExecuted = true;
            if (!fn) continue;

            dependencies.resetChanges();
            let args: any[];
            try {
              args = await isomorphoicRun(fn, [
                loaderContext.remainingRequest,
                loaderContext.previousRequest,
                currentLoaderObject.loaderItem.data,
              ]);
            } finally {
              dependencies.mergeChanges();
            }

            const hasArg = args.some((value: any) => value !== undefined);

            if (hasArg) {
              const [content, sourceMap, additionalData] = args;
              context.content = isNil(content) ? null : toBuffer(content);
              context.sourceMap = serializeObject(sourceMap);
              context.additionalData = isNil(additionalData)
                ? undefined
                : registerLoaderAdditionalData(additionalData);
              break;
            }
          }

          break;
        }
        case JsLoaderState.Normal: {
          let content: Parameters<typeof toBuffer>[0] | null | undefined =
            context.content;
          const rawSourceMap = context.sourceMap;
          let sourceMap: string | object | undefined;
          let sourceMapParsed = false;
          let additionalData = isNil(context.additionalData)
            ? undefined
            : getLoaderAdditionalData(context.additionalData);

          while (loaderContext.loaderIndex >= 0) {
            const currentLoaderObject =
              loaderContext.loaders[loaderContext.loaderIndex];
            const parallelism = enableParallelism(currentLoaderObject);

            if (currentLoaderObject.shouldYield()) break;
            if (!!parallelism !== worker) break;
            if (currentLoaderObject.normalExecuted) {
              loaderContext.loaderIndex--;
              continue;
            }

            dependencies.resetChanges();
            try {
              const cached: LoaderCacheEntry | null | undefined =
                currentLoaderObject.loaderItem.cache && loaderCache
                  ? await loaderCache.get(
                      loaderContext.loaderIndex,
                      content,
                      additionalData,
                    )
                  : undefined;
              if (cached) {
                currentLoaderObject.normalExecuted = true;
                content = cached.content;
                sourceMap = JsSourceMap.__from_binding(cached.sourceMap);
                sourceMapParsed = true;
                loaderContext.loaderIndex--;
                continue;
              }

              await loadLoader(currentLoaderObject, compiler);
              const fn = currentLoaderObject.normal;
              // If parallelism is enabled,
              // we delegate the current loader to use the runner in worker.
              currentLoaderObject.normalExecuted = true;
              if (!fn) continue;

              // Parse source map lazily only when a JavaScript loader consumes it.
              if (!sourceMapParsed) {
                sourceMap = JsSourceMap.__from_binding(rawSourceMap);
                sourceMapParsed = true;
              }

              [content, sourceMap, additionalData] = await isomorphoicRun(fn, [
                content,
                sourceMap,
                additionalData,
              ]);

              if (cached === null) {
                await loaderCache?.store(
                  loaderContext.loaderIndex,
                  content,
                  JsSourceMap.__to_binding(sourceMap),
                  additionalData,
                );
              }
            } finally {
              dependencies.mergeChanges();
            }
          }

          context.content = isNil(content) ? null : toBuffer(content);
          context.sourceMap = sourceMapParsed
            ? JsSourceMap.__to_binding(sourceMap)
            : rawSourceMap;
          context.additionalData = isNil(additionalData)
            ? undefined
            : registerLoaderAdditionalData(additionalData);
          context.__internal__utf8Hint = typeof content === 'string';

          break;
        }
        default:
          throw new Error(`Unexpected loader runner state: ${loaderState}`);
      }

      // update loader state
      context.loaderItems = loaderContext.loaders.map((item) =>
        LoaderObject.__to_binding(item),
      );
    } catch (e) {
      if (typeof e !== 'object' || e === null) {
        const error = new Error(
          `(Emitted value instead of an instance of Error) ${e}`,
        );
        error.name = 'NonErrorEmittedError';
        context.__internal__error = error;
      } else {
        context.__internal__error = e as RspackError;
      }
    }
    if (traceData) {
      JavaScriptTracer.endAsync({
        name: 'run_js_loaders',
        uuid: traceData.uuid,
        args: traceData.args,
      });
    }

    if (compiler.options?.cache && !worker) {
      commitCustomFieldsToRust(context._module.buildInfo);
    }

    return context;
  };
  return executeLoaders();
}

export function runLoaders(
  compiler: Compiler,
  context: JsLoaderContext,
  worker = false,
): Promise<JsLoaderContext> {
  return runLoadersInternal(compiler, context, worker);
}

/** Builds only the Compiler surface consumed by the shared loader-context implementation. */
export function createWorkerLoaderCompiler(
  context: JsLoaderContext,
  task: {
    getCompilation(): any;
    getResolver(options?: any): any;
    log(name: string, type: string, message?: string): void;
  },
): Compiler {
  const currentLoader = context.loaderItems[context.loaderIndex];
  const optionsHandle = currentLoader?.optionsHandle;
  const mainInputFileSystem =
    optionsHandle === null || optionsHandle === undefined
      ? null
      : getLoaderInputFileSystem(optionsHandle);
  if (!mainInputFileSystem) {
    throw new Error('Parallel loader input file system bridge is unavailable');
  }
  const compilerBridge = getLoaderCompilerBridge(optionsHandle!);

  const resolver = new Resolver(task.getResolver());
  const resolverFacade: any = {
    resolve: resolver.resolve.bind(resolver),
    resolveSync: resolver.resolveSync.bind(resolver),
  };
  resolverFacade.withOptions = (options: any) => {
    const child = new Resolver(task.getResolver(options));
    return {
      resolve: child.resolve.bind(child),
      resolveSync: child.resolveSync.bind(child),
      withOptions: resolverFacade.withOptions,
    };
  };

  const createLogger = (name: string): any => {
    const logger: any = {};
    for (const type of [
      'error',
      'warn',
      'info',
      'log',
      'debug',
      'trace',
      'group',
      'groupCollapsed',
      'groupEnd',
      'clear',
      'status',
    ]) {
      logger[type] = (...args: any[]) =>
        task.log(name, type, args.length ? format(...args) : undefined);
    }
    logger.getChildLogger = (childName: string) =>
      createLogger(`${name}/${childName}`);
    return logger;
  };

  const createStats = (metadata: any) => ({
    ...metadata,
    atime: new Date(metadata.atimeMs),
    mtime: new Date(metadata.mtimeMs),
    ctime: new Date(metadata.ctimeMs),
    isFile: () => metadata.isFile,
    isDirectory: () => metadata.isDirectory,
    isSymbolicLink: () => metadata.isSymlink,
  });
  const inputFileSystem = {
    readFile(
      path: string,
      callback: (error: Error | null, value?: Buffer) => void,
    ) {
      mainInputFileSystem.readFile(path, (error: Error | null, value?: any) =>
        callback(error, value === undefined ? undefined : Buffer.from(value)),
      );
    },
    readFileSync: (path: string) =>
      Buffer.from(mainInputFileSystem.readFileSync(path)),
    readdir(
      path: string,
      callback: (error: Error | null, value?: string[]) => void,
    ) {
      mainInputFileSystem.readdir(path, callback);
    },
    readdirSync: (path: string) => mainInputFileSystem.readdirSync(path),
    stat(path: string, callback: (error: Error | null, value?: any) => void) {
      mainInputFileSystem.stat(path, (error: Error | null, value?: any) =>
        callback(error, value ? createStats(value) : undefined),
      );
    },
    statSync: (path: string) => createStats(mainInputFileSystem.statSync(path)),
  };

  const outputOptions = {
    hashFunction: 'xxhash64',
    hashSalt: undefined,
    hashDigest: 'hex',
    hashDigestLength: 16,
    environment: {},
  };
  const compiler = {
    context: compilerBridge.context,
    root: {},
    options: {
      mode: compilerBridge.mode,
      devtool: false,
      loader: compilerBridge.loader,
      cache: false,
      experiments: { css: true },
      output: outputOptions,
    },
    inputFileSystem,
    resolverFactory: { get: () => resolverFacade },
    __internal__ruleSet: {
      references: {
        get: (ident: string) => compilerBridge.getLoaderOptionsByIdent(ident),
      },
    },
    __internal__takeModuleExecutionResult: (id: number) =>
      compilerBridge.takeModuleExecutionResult(id),
  } as unknown as Compiler;
  const compilation = new Compilation(compiler, task.getCompilation());
  compilation.getLogger = (name: string | (() => string)) =>
    createLogger(typeof name === 'function' ? name() : name);
  Object.defineProperty(compiler, '_lastCompilation', {
    value: compilation,
  });
  return compiler;
}
