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
import type { Compilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import {
  BUILTIN_LOADER_PREFIX,
  type Diagnostic,
  isUseSimpleSourceMap,
  isUseSourceMap,
  type LoaderContext,
} from '../config/adapterRuleUse';
import { NormalModule } from '../NormalModule';
import type { ResolveContext } from '../Resolver';
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
  parseResourceWithoutFragment,
} from '../util/identifier';
import { memoize } from '../util/memoize';
import { ModuleError, ModuleWarning } from './ModuleError';
import * as pool from './service';
import { type HandleIncomingRequest, RequestType } from './service';
import {
  convertArgs,
  extractLoaderName,
  loadLoader,
  runSyncOrAsync,
} from './utils';

const LOADER_PROCESS_NAME = 'Loader Analysis';

function createLoaderObject(
  loader: JsLoaderItem,
  compiler: Compiler,
): LoaderObject {
  const obj: any = {
    path: null,
    query: null,
    fragment: null,
    options: null,
    ident: null,
    normal: null,
    pitch: null,
    raw: null,
    data: null,
    pitchExecuted: false,
    normalExecuted: false,
  };
  Object.defineProperty(obj, 'request', {
    enumerable: true,
    get: () =>
      obj.path.replace(/#/g, '\u200b#') +
      obj.query.replace(/#/g, '\u200b#') +
      obj.fragment,
    set: (value: JsLoaderItem) => {
      const splittedRequest = parseResourceWithoutFragment(value.loader);
      obj.path = splittedRequest.path;
      obj.query = splittedRequest.query;
      obj.fragment = '';
      obj.options =
        obj.options === null
          ? splittedRequest.query
            ? splittedRequest.query.slice(1)
            : undefined
          : obj.options;

      if (typeof obj.options === 'string' && obj.options[0] === '?') {
        const ident = obj.options.slice(1);
        if (ident === '[[missing ident]]') {
          throw new Error(
            'No ident is provided by referenced loader. ' +
              'When using a function for Rule.use in config you need to ' +
              "provide an 'ident' property for referenced loader options.",
          );
        }
        obj.options = compiler.__internal__ruleSet.references.get(ident);
        if (obj.options === undefined) {
          throw new Error('Invalid ident is provided by referenced loader');
        }
        obj.ident = ident;
      }

      // CHANGE: `rspack_core` returns empty string for `undefined` type.
      // Comply to webpack test case: tests/webpack-test/cases/loaders/cjs-loader-type/index.js
      obj.type = value.type === '' ? undefined : value.type;
      if (obj.options === null) obj.query = '';
      else if (obj.options === undefined) obj.query = '';
      else if (typeof obj.options === 'string') obj.query = `?${obj.options}`;
      else if (obj.ident) obj.query = `??${obj.ident}`;
      else if (typeof obj.options === 'object' && obj.options.ident)
        obj.query = `??${obj.options.ident}`;
      else obj.query = `?${JSON.stringify(obj.options)}`;
    },
  });
  obj.request = loader;
  if (Object.preventExtensions) {
    Object.preventExtensions(obj);
  }
  return obj;
}

export class LoaderObject {
  request: string;
  path: string;
  query: string;
  fragment: string;
  options?: string | object;
  ident: string;
  normal?: Function;
  pitch?: Function;
  raw?: boolean;
  type?: 'module' | 'commonjs';
  parallel?: boolean | { maxWorkers?: number };
  /**
   * @internal This field is rspack internal. Do not edit.
   */
  loaderItem: JsLoaderItem;

  constructor(loaderItem: JsLoaderItem, compiler: Compiler) {
    const {
      request,
      path,
      query,
      fragment,
      options,
      ident,
      normal,
      pitch,
      raw,
      type,
    } = createLoaderObject(loaderItem, compiler);
    this.request = request;
    this.path = path;
    this.query = query;
    this.fragment = fragment;
    this.options = options;
    this.ident = ident;
    this.normal = normal;
    this.pitch = pitch;
    this.raw = raw;
    this.type = type;
    this.parallel = ident
      ? compiler.__internal__ruleSet.references.get(`${ident}$$parallelism`)
      : false;
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
  static __from_binding(map?: Buffer) {
    return isNil(map) ? undefined : toObject(map);
  }

  static __to_binding(map?: object) {
    return serializeObject(map);
  }
}

function toLoaderContent(
  content: string | Buffer | Uint8Array | null | undefined,
) {
  if (isNil(content)) return null;
  return typeof content === 'string' ? content : toBuffer(content);
}

function materializeContextValueProperties(context: JsLoaderContext) {
  Object.defineProperties(context, {
    content: {
      configurable: true,
      enumerable: true,
      value: context.content,
      writable: true,
    },
    sourceMap: {
      configurable: true,
      enumerable: true,
      value: context.sourceMap,
      writable: true,
    },
    additionalData: {
      configurable: true,
      enumerable: true,
      value: context.additionalData,
      writable: true,
    },
    loaderItems: {
      configurable: true,
      enumerable: true,
      value: context.loaderItems,
      writable: true,
    },
    __internal__error: {
      configurable: true,
      enumerable: true,
      value: undefined,
      writable: true,
    },
    __internal__utf8Hint: {
      configurable: true,
      enumerable: true,
      value: undefined,
      writable: true,
    },
  });
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

export async function runLoaders(
  compiler: Compiler,
  context: JsLoaderContext,
): Promise<JsLoaderContext> {
  const loaderState = context.loaderState;
  const pitch = loaderState === JsLoaderState.Pitching;

  const resource = context.resource;
  const module = context._module;
  const uuid = JavaScriptTracer.uuid();

  JavaScriptTracer.startAsync({
    name: 'run_js_loaders',
    processName: LOADER_PROCESS_NAME,
    uuid,
    ph: 'b',
    args: {
      is_pitch: pitch,
      resource: resource,
    },
  });
  materializeContextValueProperties(context);
  /// Construct `loaderContext`
  const loaderContext = {} as LoaderContext;
  let contextActive = true;
  const assertContextActive = () => {
    if (!contextActive) {
      throw new Error(
        'Loader context is no longer valid after the loader runner has finished',
      );
    }
  };

  loaderContext.loaders = context.loaderItems.map((item) => {
    return LoaderObject.__from_binding(item, compiler);
  });

  loaderContext.hot = context.hot;
  let resourceDataOverride:
    | { path: string | undefined; query: string; fragment: string }
    | undefined;
  let contextOverride: string | null | undefined;
  const getResourcePath = () => {
    assertContextActive();
    return resourceDataOverride
      ? resourceDataOverride.path
      : context.resourcePath;
  };
  const getResourceQuery = () => {
    assertContextActive();
    return resourceDataOverride
      ? resourceDataOverride.query
      : context.resourceQuery || '';
  };
  const getResourceFragment = () => {
    assertContextActive();
    return resourceDataOverride
      ? resourceDataOverride.fragment
      : context.resourceFragment || '';
  };
  const ensureResourceDataOverride = () => {
    assertContextActive();
    if (!resourceDataOverride) {
      resourceDataOverride = {
        path: context.resourcePath,
        query: context.resourceQuery || '',
        fragment: context.resourceFragment || '',
      };
    }
    return resourceDataOverride;
  };
  loaderContext.dependency = loaderContext.addDependency =
    function addDependency(file) {
      assertContextActive();
      context.addDependency(file);
    };
  loaderContext.addContextDependency = function addContextDependency(dep) {
    assertContextActive();
    context.addContextDependency(dep);
  };
  loaderContext.addMissingDependency = function addMissingDependency(dep) {
    assertContextActive();
    context.addMissingDependency(dep);
  };
  loaderContext.addBuildDependency = function addBuildDependency(file) {
    assertContextActive();
    context.addBuildDependency(file);
  };
  loaderContext.getDependencies = function getDependencies() {
    assertContextActive();
    return context.getDependencies();
  };
  loaderContext.getContextDependencies = function getContextDependencies() {
    assertContextActive();
    return context.getContextDependencies();
  };
  loaderContext.getMissingDependencies = function getMissingDependencies() {
    assertContextActive();
    return context.getMissingDependencies();
  };
  loaderContext.clearDependencies = function clearDependencies() {
    assertContextActive();
    context.clearDependencies();
  };

  loaderContext.importModule = function importModule(
    request,
    userOptions,
    callback,
  ) {
    assertContextActive();
    JavaScriptTracer.startAsync({
      name: 'importModule',
      processName: LOADER_PROCESS_NAME,

      uuid,
      args: {
        is_pitch: pitch,
        resource: resource,
      },
    });
    const options = userOptions ? userOptions : {};
    const context = loaderContext;
    function finalCallback(
      onError: (err: Error) => void,
      onDone: (res: any) => void,
    ) {
      return function (err?: Error, res?: any) {
        if (err) {
          JavaScriptTracer.endAsync({
            name: 'importModule',
            processName: LOADER_PROCESS_NAME,
            uuid,
            args: {
              is_pitch: pitch,
              resource: resource,
            },
          });
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
          JavaScriptTracer.endAsync({
            name: 'importModule',
            processName: LOADER_PROCESS_NAME,
            uuid,
            args: {
              is_pitch: pitch,
              resource: resource,
            },
          });
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
      assertContextActive();
      if (!resourceDataOverride) return resource || undefined;
      const resourcePath = resourceDataOverride.path;
      if (resourcePath === undefined) return undefined;
      return (
        resourcePath.replace(/#/g, '\u200b#') +
        resourceDataOverride.query.replace(/#/g, '\u200b#') +
        resourceDataOverride.fragment
      );
    },
    set: (value) => {
      assertContextActive();
      const splittedResource = value && parsePathQueryFragment(value);
      resourceDataOverride = splittedResource
        ? {
            path: splittedResource.path,
            query: splittedResource.query,
            fragment: splittedResource.fragment,
          }
        : { path: undefined, query: '', fragment: '' };
    },
  });
  Object.defineProperty(loaderContext, 'resourcePath', {
    enumerable: true,
    get: () => getResourcePath(),
    set: (value) => {
      ensureResourceDataOverride().path = value;
    },
  });
  Object.defineProperty(loaderContext, 'resourceQuery', {
    enumerable: true,
    get: () => getResourceQuery(),
    set: (value) => {
      ensureResourceDataOverride().query = value || '';
    },
  });
  Object.defineProperty(loaderContext, 'resourceFragment', {
    enumerable: true,
    get: () => getResourceFragment(),
    set: (value) => {
      ensureResourceDataOverride().fragment = value || '';
    },
  });
  Object.defineProperty(loaderContext, 'context', {
    enumerable: true,
    get: () => {
      assertContextActive();
      return contextOverride === undefined
        ? context.context || null
        : contextOverride;
    },
    set: (value) => {
      assertContextActive();
      contextOverride = value;
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
    : (module.useSourceMap ?? false);
  loaderContext.mode = compiler.options.mode;
  Object.assign(loaderContext, compiler.options.loader);

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
    assertContextActive();
    getResolver().resolve({}, context, request, getResolveContext(), callback);
  };

  loaderContext.getResolve = function getResolve(options) {
    assertContextActive();
    const resolver = getResolver();
    const child = options ? resolver.withOptions(options) : resolver;
    return (context, request, callback) => {
      assertContextActive();
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
    };
  };
  loaderContext.getLogger = function getLogger(name) {
    assertContextActive();
    return compiler._lastCompilation!.getLogger(
      [name, resource].filter(Boolean).join('|'),
    );
  };
  loaderContext.rootContext = compiler.context;
  loaderContext.emitError = function emitError(e) {
    assertContextActive();
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
    assertContextActive();
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
    assertContextActive();
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
          makePathsRelative(loaderContext.context!, sourceMap, compiler),
        );
      }

      if (loaderContext.sourceMap) {
        source = new SourceMapSource(
          content,
          name,
          makePathsRelative(loaderContext.context!, sourceMap, compiler),
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
      assertContextActive();
      const d = Object.assign({}, diagnostic, {
        message:
          diagnostic.severity === 'warning'
            ? `ModuleWarning: ${diagnostic.message}`
            : `ModuleError: ${diagnostic.message}`,
        moduleIdentifier: module.identifier(),
      });
      compiler._lastCompilation!.__internal__pushDiagnostic(
        formatDiagnostic(d),
      );
    },
  };

  const getAbsolutify = memoize(() => absolutify.bindCache(compiler.root));
  const getAbsolutifyInContext = memoize(() =>
    absolutify.bindContextCache(loaderContext.context!, compiler.root),
  );
  const getContextify = memoize(() => contextify.bindCache(compiler.root));
  const getContextifyInContext = memoize(() =>
    contextify.bindContextCache(loaderContext.context!, compiler.root),
  );

  loaderContext.utils = {
    absolutify: (context, request) => {
      return context === loaderContext.context
        ? getAbsolutifyInContext()(request)
        : getAbsolutify()(context, request);
    },
    contextify: (context, request) => {
      return context === loaderContext.context
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
  loaderContext._module = module;

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

  let compilation: Compilation | undefined = compiler._lastCompilation;
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

  /// Sync with `context`
  Object.defineProperty(loaderContext, 'loaderIndex', {
    enumerable: true,
    get: () => {
      assertContextActive();
      return context.loaderIndex;
    },
    set: (loaderIndex) => {
      assertContextActive();
      context.loaderIndex = loaderIndex;
    },
  });
  Object.defineProperty(loaderContext, 'cacheable', {
    enumerable: true,
    get: () => (cacheable?: boolean) => {
      assertContextActive();
      if (cacheable === false) {
        context.setCacheable(cacheable);
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
    assertContextActive();
    context.__internal__setParseMeta(key, value);
  };

  const getWorkerLoaderContext = () => {
    const normalModule =
      loaderContext._module instanceof NormalModule
        ? loaderContext._module
        : undefined;
    const workerLoaderContext = {
      hot: loaderContext.hot,
      context: loaderContext.context,
      resourcePath: loaderContext.resourcePath,
      resourceQuery: loaderContext.resourceQuery,
      resourceFragment: loaderContext.resourceFragment,
      resource: loaderContext.resource,
      mode: loaderContext.mode,
      sourceMap: loaderContext.sourceMap,
      rootContext: loaderContext.rootContext,
      loaderIndex: loaderContext.loaderIndex,
      loaders: loaderContext.loaders.map((item) => {
        let options = item.options;
        // Do not pass options into worker, if it's not prepared to be executed
        // in the worker thread.
        //
        // Aligns yielding strategy within the worker.
        if (!item.parallel || item.request.startsWith(BUILTIN_LOADER_PREFIX)) {
          options = undefined;
        }
        return {
          ...item,
          options,
          pitch: undefined,
          normal: undefined,
          normalExecuted: item.normalExecuted,
          pitchExecuted: item.pitchExecuted,
        };
      }),

      __internal__workerInfo: {
        hashFunction: compiler._lastCompilation!.outputOptions.hashFunction!,
      },
      _compiler: {
        options: {
          experiments: {
            css: true,
          },
        },
      },
      _compilation: {
        options: {
          output: {
            // css-loader
            environment: compiler._lastCompilation!.outputOptions.environment,
          },
        },
        // css-loader
        outputOptions: {
          hashSalt: compiler._lastCompilation!.outputOptions.hashSalt,
          hashFunction: compiler._lastCompilation!.outputOptions.hashFunction,
          hashDigest: compiler._lastCompilation!.outputOptions.hashDigest,
          hashDigestLength:
            compiler._lastCompilation!.outputOptions.hashDigestLength,
        },
      },
      _module: {
        type: loaderContext._module.type,
        identifier: loaderContext._module.identifier(),
        matchResource: normalModule?.matchResource,
        request: normalModule?.request,
        userRequest: normalModule?.userRequest,
        rawRequest: normalModule?.rawRequest,
      },
    } as any;
    Object.assign(workerLoaderContext, compiler.options.loader);
    return workerLoaderContext;
  };

  const getWorkerLoaderHandlers = function (): {
    handleIncomingRequest: HandleIncomingRequest;
  } {
    return {
      handleIncomingRequest(requestType, ...args) {
        switch (requestType) {
          case RequestType.AddDependency: {
            loaderContext.addDependency(args[0]);
            break;
          }
          case RequestType.AddContextDependency: {
            loaderContext.addContextDependency(args[0]);
            break;
          }
          case RequestType.AddMissingDependency: {
            loaderContext.addMissingDependency(args[0]);
            break;
          }
          case RequestType.AddBuildDependency: {
            loaderContext.addBuildDependency(args[0]);
            break;
          }
          case RequestType.GetDependencies: {
            return loaderContext.getDependencies();
          }
          case RequestType.GetContextDependencies: {
            return loaderContext.getContextDependencies();
          }
          case RequestType.GetMissingDependencies: {
            return loaderContext.getMissingDependencies();
          }
          case RequestType.ClearDependencies: {
            loaderContext.clearDependencies();
            break;
          }
          case RequestType.Resolve: {
            return new Promise((resolve, reject) => {
              loaderContext.resolve(args[0], args[1], (err, result) => {
                if (err) reject(err);
                else resolve(result);
              });
            });
          }
          case RequestType.GetResolve: {
            return new Promise((resolve, reject) => {
              loaderContext.getResolve(args[0])(
                args[1],
                args[2],
                (err, result) => {
                  if (err) reject(err);
                  else resolve(result);
                },
              );
            });
          }
          case RequestType.GetLogger: {
            const [type, name, arg] = args;
            (loaderContext.getLogger(name) as any)[type](...arg);
            break;
          }
          case RequestType.EmitError: {
            const workerError = args[0];
            const error = new Error(workerError.message);
            error.stack = workerError.stack;
            error.name = workerError.name;
            loaderContext.emitError(error);
            break;
          }
          case RequestType.EmitWarning: {
            const workerError = args[0];
            const error = new Error(workerError.message);
            error.stack = workerError.stack;
            error.name = workerError.name;
            loaderContext.emitWarning(error);
            break;
          }
          case RequestType.EmitFile: {
            const [name, content, sourceMap, assetInfo] = args;
            loaderContext.emitFile(name, content, sourceMap, assetInfo);
            break;
          }
          case RequestType.EmitDiagnostic: {
            const diagnostic = args[0];
            loaderContext.experiments.emitDiagnostic(diagnostic);
            break;
          }
          case RequestType.SetCacheable: {
            const cacheable = args[0];
            loaderContext.cacheable(cacheable);
            break;
          }
          case RequestType.ImportModule: {
            return loaderContext.importModule(args[0], args[1]);
          }
          case RequestType.UpdateLoaderObjects: {
            const updates = args[0];
            loaderContext.loaders = loaderContext.loaders.map((item, index) => {
              const update = updates[index];
              item.loaderItem.data = update.data;
              if (update.pitchExecuted) {
                item.pitchExecuted = true;
              }
              if (update.normalExecuted) {
                item.normalExecuted = true;
              }
              return item;
            });
            break;
          }
          case RequestType.CompilationGetPath: {
            const filename = args[0];
            const data = args[1];
            return compiler._lastCompilation!.getPath(filename, data);
          }
          case RequestType.CompilationGetPathWithInfo: {
            const filename = args[0];
            const data = args[1];
            return compiler._lastCompilation!.getPathWithInfo(filename, data);
          }
          case RequestType.CompilationGetAssetPath: {
            const filename = args[0];
            const data = args[1];
            return compiler._lastCompilation!.getAssetPath(filename, data);
          }
          case RequestType.CompilationGetAssetPathWithInfo: {
            const filename = args[0];
            const data = args[1];
            return compiler._lastCompilation!.getAssetPathWithInfo(
              filename,
              data,
            );
          }
          default: {
            throw new Error(`Unknown request type: ${requestType}`);
          }
        }
      },
    };
  };

  const enableParallelism = (currentLoaderObject: any) => {
    return currentLoaderObject?.parallel;
  };

  const isomorphoicRun = async (fn: Function, args: any[]) => {
    const currentLoaderObject = getCurrentLoader(loaderContext);
    const parallelism = enableParallelism(currentLoaderObject);
    const pitch = loaderState === JsLoaderState.Pitching;
    const loaderName = extractLoaderName(currentLoaderObject!.request);
    let result: any;
    JavaScriptTracer.startAsync({
      name: loaderName,
      trackName: loaderName,
      processName: LOADER_PROCESS_NAME,
      uuid,
      args: {
        is_pitch: pitch,
        resource: resource,
      },
    });
    if (parallelism) {
      result =
        (await pool.run(
          loaderName,
          {
            loaderContext: getWorkerLoaderContext(),
            loaderState,
            args,
          },
          getWorkerLoaderHandlers(),
          typeof currentLoaderObject?.parallel === 'object'
            ? currentLoaderObject.parallel
            : undefined,
        )) || [];
    } else {
      if (loaderState === JsLoaderState.Normal)
        convertArgs(args, !!currentLoaderObject?.raw);
      result = (await runSyncOrAsync(fn, loaderContext, args)) || [];
    }
    JavaScriptTracer.endAsync({
      name: loaderName,
      trackName: loaderName,
      processName: LOADER_PROCESS_NAME,
      uuid,
      args: {
        is_pitch: pitch,
        resource: resource,
      },
    });
    return result;
  };

  try {
    switch (loaderState) {
      case JsLoaderState.Pitching: {
        while (loaderContext.loaderIndex < loaderContext.loaders.length) {
          const currentLoaderObject =
            loaderContext.loaders[loaderContext.loaderIndex];
          const parallelism = enableParallelism(currentLoaderObject);

          if (currentLoaderObject.shouldYield()) break;
          if (currentLoaderObject.pitchExecuted) {
            loaderContext.loaderIndex += 1;
            continue;
          }

          await loadLoader(currentLoaderObject, compiler);
          const fn = currentLoaderObject.pitch;
          // If parallelism is enabled,
          // we delegate the current loader to use the runner in worker.
          if (!parallelism || !fn) {
            currentLoaderObject.pitchExecuted = true;
          }
          if (!fn) continue;

          const args = await isomorphoicRun(fn, [
            loaderContext.remainingRequest,
            loaderContext.previousRequest,
            currentLoaderObject.loaderItem.data,
          ]);

          const hasArg = args.some((value: any) => value !== undefined);

          if (hasArg) {
            const [content, sourceMap, additionalData] = args;
            context.content = toLoaderContent(content);
            context.sourceMap = serializeObject(sourceMap);
            context.additionalData = additionalData || undefined;
            break;
          }
        }

        break;
      }
      case JsLoaderState.Normal: {
        let content = context.content;
        let sourceMap = JsSourceMap.__from_binding(context.sourceMap);
        let additionalData = context.additionalData;

        while (loaderContext.loaderIndex >= 0) {
          const currentLoaderObject =
            loaderContext.loaders[loaderContext.loaderIndex];
          const parallelism = enableParallelism(currentLoaderObject);

          if (currentLoaderObject.shouldYield()) break;
          if (currentLoaderObject.normalExecuted) {
            loaderContext.loaderIndex--;
            continue;
          }

          await loadLoader(currentLoaderObject, compiler);
          const fn = currentLoaderObject.normal;
          // If parallelism is enabled,
          // we delegate the current loader to use the runner in worker.
          if (!parallelism || !fn) {
            currentLoaderObject.normalExecuted = true;
          }
          if (!fn) continue;
          [content, sourceMap, additionalData] = await isomorphoicRun(fn, [
            content,
            sourceMap,
            additionalData,
          ]);
        }

        context.content = toLoaderContent(content);
        context.sourceMap = JsSourceMap.__to_binding(sourceMap);
        context.additionalData = additionalData || undefined;
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
  JavaScriptTracer.endAsync({
    name: 'run_js_loaders',
    uuid,
    args: {
      is_pitch: pitch,
      resource: resource,
    },
  });

  contextActive = false;
  if (compiler.options?.cache) {
    commitCustomFieldsToRust(module.buildInfo);
  }

  return context;
}

const PATH_QUERY_FRAGMENT_REGEXP =
  /^((?:\u200b.|[^?#\u200b])*)(\?(?:\u200b.|[^#\u200b])*)?(#.*)?$/;

export function parsePathQueryFragment(str: string): {
  path: string;
  query: string;
  fragment: string;
} {
  const match = PATH_QUERY_FRAGMENT_REGEXP.exec(str);
  return {
    path: match?.[1].replace(/\u200b(.)/g, '$1') || '',
    query: match?.[2] ? match[2].replace(/\u200b(.)/g, '$1') : '',
    fragment: match?.[3] || '',
  };
}
