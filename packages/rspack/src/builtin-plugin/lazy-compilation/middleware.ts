import type { IncomingMessage, ServerResponse } from 'node:http';
import { createRequire } from 'node:module';
import { type Compiler, MultiCompiler } from '../..';
import type { LazyCompilationOptions } from '../../config';
import type { ExternalItem, Externals } from '../../config';
import type { DevServerMiddlewareHandler } from '../../config/devServer';
import { BuiltinLazyCompilationPlugin } from './lazyCompilation';

const require = createRequire(import.meta.url);

export const LAZY_COMPILATION_PREFIX = '/_rspack/lazy/trigger';

const LAZY_COMPILATION_IDLE_TIMEOUT = 120_000;

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
): DevServerMiddlewareHandler => {
  if (compiler instanceof MultiCompiler) {
    const middlewareByCompiler: Map<string, DevServerMiddlewareHandler> =
      new Map();

    let i = 0;

    for (const c of compiler.compilers) {
      if (!c.options.lazyCompilation) {
        continue;
      }

      const options = {
        ...c.options.lazyCompilation,
      };

      const prefix = options.prefix || LAZY_COMPILATION_PREFIX;
      options.prefix = `${prefix}__${i++}`;
      const activeModules = new Map<string, number>();
      const legacyModules = new Map<string, NodeJS.Timeout>();

      middlewareByCompiler.set(
        options.prefix,
        lazyCompilationMiddlewareInternal(
          c,
          activeModules,
          legacyModules,
          options.prefix,
        ),
      );

      applyPlugin(c, options, activeModules, legacyModules);
    }

    const keys = [...middlewareByCompiler.keys()];
    return (req: IncomingMessage, res: ServerResponse, next: () => void) => {
      const key = keys.find(
        (key) => req.url === key || req.url?.startsWith(`${key}?`),
      );
      if (!key) {
        return next?.();
      }

      const middleware = middlewareByCompiler.get(key);

      return middleware?.(req, res, next);
    };
  }

  if (!compiler.options.lazyCompilation) {
    return noop;
  }

  const activeModules = new Map<string, number>();
  const legacyModules = new Map<string, NodeJS.Timeout>();

  const options = {
    ...compiler.options.lazyCompilation,
  };

  applyPlugin(compiler, options, activeModules, legacyModules);

  const lazyCompilationPrefix = options.prefix || LAZY_COMPILATION_PREFIX;
  return lazyCompilationMiddlewareInternal(
    compiler,
    activeModules,
    legacyModules,
    lazyCompilationPrefix,
  );
};

function applyPlugin(
  compiler: Compiler,
  options: LazyCompilationOptions,
  activeModules: Map<string, number>,
  legacyModules: Map<string, NodeJS.Timeout>,
) {
  const plugin = new BuiltinLazyCompilationPlugin(
    () => {
      const modules = new Set(activeModules.keys());
      for (const key of legacyModules.keys()) {
        modules.add(key);
      }
      return modules;
    },
    options.entries ?? true,
    options.imports ?? true,
    `${options.client || getDefaultClient(compiler)}?${encodeURIComponent(getFullServerUrl(options))}`,
    collectReservedExternals(compiler.options.externals),
    options.test,
  );
  plugin.apply(compiler);
}

// Statically-declared external requests to reserve in the closure-library wrapper
// (externals live on the JS options, not Rust's `CompilerOptions`).
function collectReservedExternals(externals: Externals | undefined): string[] {
  const requests = new Set<string>();
  const visit = (item: ExternalItem) => {
    if (typeof item === 'string') {
      requests.add(item);
      return;
    }
    if (!item || typeof item !== 'object' || item instanceof RegExp) {
      return;
    }
    for (const [request, value] of Object.entries(item)) {
      // `false` opts out of externalization; every other key mirrors `ExternalsPlugin`.
      if (value === false) {
        continue;
      }
      requests.add(request);
    }
  };
  if (Array.isArray(externals)) {
    for (const item of externals) {
      visit(item);
    }
  } else if (externals !== undefined) {
    visit(externals);
  }
  return [...requests];
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
  compiler: Compiler,
  activeModules: Map<string, number>,
  legacyModules: Map<string, NodeJS.Timeout>,
  lazyCompilationPrefix: string,
): DevServerMiddlewareHandler => {
  const logger = compiler.getInfrastructureLogger('LazyCompilation');
  const idleTimers = new Set<NodeJS.Timeout>();
  const activeResponses = new Set<ServerResponse>();
  let isClosing = false;

  compiler.hooks.shutdown.tap('LazyCompilationMiddleware', () => {
    isClosing = true;
    for (const timer of idleTimers) {
      clearTimeout(timer);
    }
    idleTimers.clear();
    for (const response of activeResponses) {
      response.destroy();
    }
    activeResponses.clear();
    activeModules.clear();
    legacyModules.clear();
  });

  return async (
    req: IncomingMessage,
    res: ServerResponse,
    next?: () => void,
  ) => {
    if (!req.url?.startsWith(lazyCompilationPrefix) || req.method !== 'POST') {
      return next?.();
    }

    let modules: string[] = [];
    try {
      modules = await readModuleIdsFromBody(req);
    } catch (err) {
      logger.error(`Failed to parse request body: ${err}`);
      res.writeHead(400);
      res.end('Bad Request');
      return;
    }

    if (isClosing) {
      res.writeHead(503);
      res.end('Compiler is shutting down');
      return;
    }

    const keys = new Set(modules);

    if (!req.headers.accept?.includes('text/event-stream')) {
      let moduleActivated = false;
      for (const key of keys) {
        const previousTimer = legacyModules.get(key);
        const activated = activeModules.has(key) || previousTimer !== undefined;
        if (previousTimer) {
          clearTimeout(previousTimer);
          idleTimers.delete(previousTimer);
        }

        const timer = setTimeout(() => {
          idleTimers.delete(timer);
          if (legacyModules.get(key) === timer) {
            legacyModules.delete(key);
            if (!activeModules.has(key)) {
              logger.log(
                `${key} is no longer in use. Next compilation will skip this module.`,
              );
            }
          }
        }, LAZY_COMPILATION_IDLE_TIMEOUT);
        timer.unref?.();
        idleTimers.add(timer);
        legacyModules.set(key, timer);

        if (!activated) {
          logger.log(`${key} is now in use and will be compiled.`);
          moduleActivated = true;
        }
      }

      if (moduleActivated && compiler.watching) {
        compiler.watching.invalidate();
      }

      res.writeHead(200);
      res.end('\n');
      return;
    }

    req.socket.on('close', () => {
      if (isClosing) {
        return;
      }

      // Keep recently disconnected clients active long enough to reconnect
      // without turning an HMR update into another lazy activation rebuild.
      const timer = setTimeout(() => {
        idleTimers.delete(timer);
        for (const key of keys) {
          const oldValue = activeModules.get(key) || 0;
          if (oldValue <= 1) {
            activeModules.delete(key);
            if (oldValue === 1 && !legacyModules.has(key)) {
              logger.log(
                `${key} is no longer in use. Next compilation will skip this module.`,
              );
            }
          } else {
            activeModules.set(key, oldValue - 1);
          }
        }
      }, LAZY_COMPILATION_IDLE_TIMEOUT);
      timer.unref?.();
      idleTimers.add(timer);
    });

    req.socket.setNoDelay?.(true);
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': '*',
      'Access-Control-Allow-Headers': '*',
    });
    activeResponses.add(res);
    res.on('close', () => {
      activeResponses.delete(res);
    });
    res.write('\n');

    let moduleActivated = false;
    for (const key of keys) {
      const oldValue = activeModules.get(key) || 0;
      activeModules.set(key, oldValue + 1);
      if (oldValue === 0 && !legacyModules.has(key)) {
        logger.log(`${key} is now in use and will be compiled.`);
        moduleActivated = true;
      }
    }

    if (moduleActivated && compiler.watching) {
      compiler.watching.invalidate();
    }
  };
};
