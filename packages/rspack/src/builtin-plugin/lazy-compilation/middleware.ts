import type { IncomingMessage, ServerResponse } from "node:http";
import type { Compiler, LazyCompilationOptions } from "../..";
import type { Module } from "../../Module";
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
	compiler: Compiler,
	userOptions: LazyCompilationOptions | boolean = {}
): Middleware => {
	if (userOptions === false) {
		return noop;
	}

	const options = userOptions === true ? {} : userOptions;
	const activeModules: Map<string, boolean> = new Map();
	const filesByKey: Map<string, string> = new Map();
	const lazyCompilationPrefix = options.prefix || LAZY_COMPILATION_PREFIX;

	new BuiltinLazyCompilationPlugin(
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
		typeof options.test === "function"
			? module => {
					const test = options.test as (module: Module) => boolean;
					return test(module);
				}
			: options.test
	).apply(compiler);

	return lazyCompilationMiddlewareInternal(
		compiler,
		activeModules,
		filesByKey,
		lazyCompilationPrefix
	);
};

// used for reuse code, do not export this
const lazyCompilationMiddlewareInternal = (
	compiler: Compiler,
	activeModules: Map<string, boolean>,
	filesByKey: Map<string, string>,
	lazyCompilationPrefix: string
) => {
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
