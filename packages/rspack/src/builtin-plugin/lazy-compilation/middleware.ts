import type { IncomingMessage, ServerResponse } from 'node:http';
import { type Compiler, MultiCompiler } from '../..';
import type { LazyCompilationOptions } from '../../config';
import type { MiddlewareHandler } from '../../config/devServer';
import { deprecate } from '../../util';
import { BuiltinLazyCompilationPlugin } from './lazyCompilation';

export const LAZY_COMPILATION_PREFIX = '/lazy-compilation-using-';

const getDefaultClient = (compiler: Compiler): string =>
  require.resolve(
    `../hot/lazy-compilation-${
      compiler.options.externalsPresets.node ? 'node' : 'web'
    }.js`,
  );

const noop = (
  _req: IncomingMessage,
  _res: ServerResponse,
  next?: (err?: any) => void,
) => {
  if (typeof next === 'function') {
    next();
  }
};

const getFullServerUrl = ({ serverUrl, prefix }: LazyCompilationOptions) => {
  const lazyCompilationPrefix = prefix || LAZY_COMPILATION_PREFIX;
  if (!serverUrl) {
    return lazyCompilationPrefix;
  }
  return (
    serverUrl +
    (serverUrl.endsWith('/')
      ? lazyCompilationPrefix.slice(1)
      : lazyCompilationPrefix)
  );
};

const DEPRECATED_LAZY_COMPILATION_OPTIONS_WARN =
  'The `experiments.lazyCompilation` option is deprecated, please use the configuration top level `lazyCompilation` instead.';

const REPEAT_LAZY_COMPILATION_OPTIONS_WARN =
  'Both top-level `lazyCompilation` and `experiments.lazyCompilation` options are set. The top-level `lazyCompilation` configuration will take precedence.';

/**
 * Create a middleware that handles lazy compilation requests from the client.
 * This function returns an Express-style middleware that listens for
 * requests triggered by lazy compilation in the dev server client,
 * then invokes the Rspack compiler to compile modules on demand.
 * Use this middleware when integrating lazy compilation into a
 * custom development server instead of relying on the built-in server.
 */
export const lazyCompilationMiddleware = (
  compiler: Compiler | MultiCompiler,
): MiddlewareHandler => {
  if (compiler instanceof MultiCompiler) {
    const middlewareByCompiler: Map<string, MiddlewareHandler> = new Map();

    let i = 0;

    for (const c of compiler.compilers) {
      if (c.options.experiments.lazyCompilation) {
        if (c.name) {
          deprecate(
            `The 'experiments.lazyCompilation' option in compiler named '${c.name}' is deprecated, please use the Configuration top level 'lazyCompilation' instead.`,
          );
        } else {
          deprecate(DEPRECATED_LAZY_COMPILATION_OPTIONS_WARN);
        }
      }

      if (c.options.lazyCompilation && c.options.experiments.lazyCompilation) {
        if (c.name) {
          deprecate(
            `The top-level 'lazyCompilation' option in compiler named '${c.name}' will override the 'experiments.lazyCompilation' option.`,
          );
        } else {
          deprecate(REPEAT_LAZY_COMPILATION_OPTIONS_WARN);
        }
      }

      if (
        !c.options.lazyCompilation &&
        !c.options.experiments.lazyCompilation
      ) {
        continue;
      }

      const options = {
        ...c.options.experiments.lazyCompilation,
        ...c.options.lazyCompilation,
      };

      const prefix = options.prefix || LAZY_COMPILATION_PREFIX;
      options.prefix = `${prefix}__${i++}`;
      const activeModules = new Set<string>();

      middlewareByCompiler.set(
        options.prefix,
        lazyCompilationMiddlewareInternal(
          compiler,
          activeModules,
          options.prefix,
        ),
      );

      applyPlugin(c, options, activeModules);
    }

    const keys = [...middlewareByCompiler.keys()];
    return (req: IncomingMessage, res: ServerResponse, next: () => void) => {
      const key = keys.find((key) => req.url?.startsWith(key));
      if (!key) {
        return next?.();
      }

      const middleware = middlewareByCompiler.get(key);

      return middleware?.(req, res, next);
    };
  }

  if (compiler.options.experiments.lazyCompilation) {
    deprecate(DEPRECATED_LAZY_COMPILATION_OPTIONS_WARN);
    if (compiler.options.lazyCompilation) {
      deprecate(REPEAT_LAZY_COMPILATION_OPTIONS_WARN);
    }
  }

  if (
    !compiler.options.lazyCompilation &&
    !compiler.options.experiments.lazyCompilation
  ) {
    return noop;
  }

  const activeModules: Set<string> = new Set();

  const options = {
    // TODO: remove this when experiments.lazyCompilation is removed
    ...compiler.options.experiments.lazyCompilation,
    ...compiler.options.lazyCompilation,
  };

  applyPlugin(compiler, options, activeModules);

  const lazyCompilationPrefix = options.prefix || LAZY_COMPILATION_PREFIX;
  return lazyCompilationMiddlewareInternal(
    compiler,
    activeModules,
    lazyCompilationPrefix,
  );
};

function applyPlugin(
  compiler: Compiler,
  options: LazyCompilationOptions,
  activeModules: Set<string>,
) {
  const plugin = new BuiltinLazyCompilationPlugin(
    () => {
      const res = new Set(activeModules);
      activeModules.clear();
      return res;
    },
    options.entries ?? true,
    options.imports ?? true,
    `${options.client || getDefaultClient(compiler)}?${encodeURIComponent(getFullServerUrl(options))}`,
    options.test,
  );
  plugin.apply(compiler);
}

function readModuleIdsFromBody(
  req: IncomingMessage & { body?: unknown },
): Promise<string[]> {
  // If body is already parsed by another middleware, use it directly
  if (req.body !== undefined) {
    if (Array.isArray(req.body)) {
      return Promise.resolve(req.body);
    }
    if (typeof req.body === 'string') {
      return Promise.resolve(req.body.split('\n').filter(Boolean));
    }
    throw new Error('Invalid body type');
  }

  return new Promise((resolve, reject) => {
    if ((req as any).aborted || req.destroyed) {
      reject(new Error('Request was aborted before body could be read'));
      return;
    }

    const cleanup = () => {
      req.removeListener('data', onData);
      req.removeListener('end', onEnd);
      req.removeListener('error', onError);
      req.removeListener('close', onClose);
      req.removeListener('aborted', onAborted);
    };

    const chunks: Buffer[] = [];
    const onData = (chunk: Buffer) => {
      chunks.push(chunk);
    };

    const onEnd = () => {
      cleanup();
      // Concatenate all chunks and decode as UTF-8 to handle multibyte characters correctly
      const body = Buffer.concat(chunks).toString('utf8');
      resolve(body.split('\n').filter(Boolean));
    };

    const onError = (err: Error) => {
      cleanup();
      reject(err);
    };

    const onClose = () => {
      cleanup();
      reject(new Error('Request was closed before body could be read'));
    };

    const onAborted = () => {
      cleanup();
      reject(new Error('Request was aborted before body could be read'));
    };

    req.on('data', onData);
    req.on('end', onEnd);
    req.on('error', onError);
    req.on('close', onClose);
    req.on('aborted', onAborted);
  });
}

const lazyCompilationMiddlewareInternal = (
  compiler: Compiler | MultiCompiler,
  activeModules: Set<string>,
  lazyCompilationPrefix: string,
): MiddlewareHandler => {
  const logger = compiler.getInfrastructureLogger('LazyCompilation');

  return async (
    req: IncomingMessage,
    res: ServerResponse,
    next?: () => void,
  ) => {
    if (!req.url?.startsWith(lazyCompilationPrefix)) {
      return next?.();
    }

    let modules: string[] = [];
    if (req.method === 'POST') {
      try {
        modules = await readModuleIdsFromBody(req);
      } catch (err) {
        logger.error('Failed to parse request body: ' + err);
        res.writeHead(400);
        res.end('Bad Request');
        return;
      }
    } else {
      modules = req.url
        .slice(lazyCompilationPrefix.length)
        .split('@')
        .map(decodeURIComponent);
    }

    req.socket.setNoDelay(true);

    res.setHeader('content-type', 'text/event-stream');
    res.writeHead(200);
    res.write('\n');

    const moduleActivated = [];
    for (const key of modules) {
      const activated = activeModules.has(key);
      activeModules.add(key);
      if (!activated) {
        logger.log(`${key} is now in use and will be compiled.`);
        moduleActivated.push(key);
      }
    }

    if (moduleActivated.length && compiler.watching) {
      compiler.watching.invalidate();
    }
    if (req.method === 'POST') {
      res.end();
    }
  };
};
