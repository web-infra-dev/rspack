import type { IncomingMessage, ServerResponse } from "node:http";
import { type Compiler, MultiCompiler } from "../..";
import type { LazyCompilationOptions } from "../../config";
import type { Middleware } from "../../config/devServer";
import { BuiltinLazyCompilationPlugin } from "./lazyCompilation";

export const LAZY_COMPILATION_PREFIX = "/lazy-compilation-using-";

const getDefaultClient = (compiler: Compiler): string =>
	require.resolve(
		`../hot/lazy-compilation-${
			compiler.options.externalsPresets.node ? "node" : "web"
		}.js`
	);

const noop = (
	_req: IncomingMessage,
	_res: ServerResponse,
	next?: (err?: any) => void
) => {
	if (typeof next === "function") {
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
		(serverUrl.endsWith("/")
			? lazyCompilationPrefix.slice(1)
			: lazyCompilationPrefix)
	);
};

export const lazyCompilationMiddleware = (
	compiler: Compiler | MultiCompiler
): Middleware => {
	if (compiler instanceof MultiCompiler) {
		const middlewareByCompiler: Map<string, Middleware> = new Map();

		let i = 0;
		for (const c of compiler.compilers) {
			if (!c.options.experiments.lazyCompilation) {
				continue;
			}

			const options = {
				...c.options.experiments.lazyCompilation
			};

			const prefix = options.prefix || LAZY_COMPILATION_PREFIX;
			options.prefix = `${prefix}__${i++}`;
			const activeModules = new Map<string, boolean>();
			const filesByKey = new Map<string, string>();

			middlewareByCompiler.set(
				options.prefix,
				lazyCompilationMiddlewareInternal(
					compiler,
					activeModules,
					filesByKey,
					options.prefix
				)
			);

			applyPlugin(c, options, activeModules, filesByKey);
		}

		const keys = [...middlewareByCompiler.keys()];
		return (req: IncomingMessage, res: ServerResponse, next?: () => void) => {
			const key = keys.find(key => req.url?.startsWith(key));
			if (!key) {
				return next?.();
			}

			const middleware = middlewareByCompiler.get(key);

			return middleware?.(req, res, next);
		};
	}

	if (!compiler.options.experiments.lazyCompilation) {
		return noop;
	}

	const activeModules: Map<string, boolean> = new Map();
	const filesByKey: Map<string, string> = new Map();

	const options = {
		...compiler.options.experiments.lazyCompilation
	};
	applyPlugin(compiler, options, activeModules, filesByKey);

	const lazyCompilationPrefix = options.prefix || LAZY_COMPILATION_PREFIX;
	return lazyCompilationMiddlewareInternal(
		compiler,
		activeModules,
		filesByKey,
		lazyCompilationPrefix
	);
};

function applyPlugin(
	compiler: Compiler,
	options: LazyCompilationOptions,
	activeModules: Map<string, boolean>,
	filesByKey: Map<string, string>
) {
	const plugin = new BuiltinLazyCompilationPlugin(
		({ module, path }) => {
			const key = encodeURIComponent(
				module.replace(/\\/g, "/").replace(/@/g, "_")
			)
				// module identifier may contain query, bang(!) or split(|),
				// should do our best to ensure it's the same with which comes
				// from server url
				.replace(/%(2F|3A|24|26|2B|2C|3B|3D)/g, decodeURIComponent);
			filesByKey.set(key, path);
			const active = activeModules.get(key) === true;
			return {
				client: `${options.client || getDefaultClient(compiler)}?${encodeURIComponent(getFullServerUrl(options))}`,
				data: key,
				active
			};
		},
		// @ts-expect-error internal option
		options.cacheable ?? true,
		options.entries ?? true,
		options.imports ?? true,
		options.test
	);
	plugin.apply(compiler);
}

// used for reuse code, do not export this
const lazyCompilationMiddlewareInternal = (
	compiler: Compiler | MultiCompiler,
	activeModules: Map<string, boolean>,
	filesByKey: Map<string, string>,
	lazyCompilationPrefix: string
): Middleware => {
	const logger = compiler.getInfrastructureLogger("LazyCompilation");

	return (req: IncomingMessage, res: ServerResponse, next?: () => void) => {
		if (!req.url?.startsWith(lazyCompilationPrefix)) {
			// only handle requests that are come from lazyCompilation
			return next?.();
		}

		const keys = req.url.slice(lazyCompilationPrefix.length).split("@");
		req.socket.setNoDelay(true);

		res.setHeader("content-type", "text/event-stream");
		res.writeHead(200);
		res.write("\n");

		const moduleActivated = [];
		for (const key of keys) {
			const oldValue = activeModules.get(key) ?? false;
			activeModules.set(key, true);
			if (!oldValue) {
				logger.log(`${key} is now in use and will be compiled.`);
				moduleActivated.push(key);
			}
		}

		if (moduleActivated.length && compiler.watching) {
			const rebuiltModules = new Set(
				moduleActivated
					.map(key => {
						const filePath = filesByKey.get(key);
						if (!filePath) {
							logger.warn(`Cannot find correct file path for module ${key}`);
						}
						return filePath;
					})
					.filter(Boolean) as string[]
			);

			if (rebuiltModules.size) {
				compiler.watching.invalidateWithChangesAndRemovals(
					new Set(rebuiltModules)
				);
			}
		}
	};
};
