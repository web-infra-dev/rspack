import type { IncomingMessage, ServerResponse } from "node:http";
import { type Compiler, MultiCompiler } from "../..";
import type { LazyCompilationOptions } from "../../config";
import type { MiddlewareHandler } from "../../config/devServer";
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

const DEPRECATED_LAZY_COMPILATION_OPTIONS_WARN =
	"The `experiments.lazyCompilation` option is deprecated, please use the configuration top level `lazyCompilation` instead.";

const REPEAT_LAZY_COMPILATION_OPTIONS_WARN =
	"Both top-level `lazyCompilation` and `experiments.lazyCompilation` options are set. The top-level `lazyCompilation` configuration will take precedence.";

export const lazyCompilationMiddleware = (
	compiler: Compiler | MultiCompiler
): MiddlewareHandler => {
	if (compiler instanceof MultiCompiler) {
		const middlewareByCompiler: Map<string, MiddlewareHandler> = new Map();

		let i = 0;
		let isReportDeprecatedWarned = false;
		let isReportRepeatWarned = false;
		for (const c of compiler.compilers) {
			if (c.options.experiments.lazyCompilation) {
				if (c.name) {
					console.warn(
						`The 'experiments.lazyCompilation' option in compiler named '${c.name}' is deprecated, please use the Configuration top level 'lazyCompilation' instead.`
					);
				} else if (!isReportDeprecatedWarned) {
					console.warn(DEPRECATED_LAZY_COMPILATION_OPTIONS_WARN);
					isReportDeprecatedWarned = true;
				}
			}

			if (c.options.lazyCompilation && c.options.experiments.lazyCompilation) {
				if (c.name) {
					console.warn(
						`The top-level 'lazyCompilation' option in compiler named '${c.name}' will override the 'experiments.lazyCompilation' option.`
					);
				} else if (!isReportRepeatWarned) {
					console.warn(REPEAT_LAZY_COMPILATION_OPTIONS_WARN);
					isReportRepeatWarned = true;
				}
			}

			if (
				!c.options.lazyCompilation &&
				!c.options.experiments.lazyCompilation
			) {
				continue;
			}

			const options = {
				// TODO: remove this when experiments.lazyCompilation is removed
				...c.options.experiments.lazyCompilation,
				...c.options.lazyCompilation
			};

			const prefix = options.prefix || LAZY_COMPILATION_PREFIX;
			options.prefix = `${prefix}__${i++}`;
			const activeModules = new Set<string>();
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
		return (req: IncomingMessage, res: ServerResponse, next: () => void) => {
			const key = keys.find(key => req.url?.startsWith(key));
			if (!key) {
				return next?.();
			}

			const middleware = middlewareByCompiler.get(key);

			return middleware?.(req, res, next);
		};
	}

	if (compiler.options.experiments.lazyCompilation) {
		console.warn(DEPRECATED_LAZY_COMPILATION_OPTIONS_WARN);
		if (compiler.options.lazyCompilation) {
			console.warn(REPEAT_LAZY_COMPILATION_OPTIONS_WARN);
		}
	}

	if (
		!compiler.options.lazyCompilation &&
		!compiler.options.experiments.lazyCompilation
	) {
		return noop;
	}

	const activeModules: Set<string> = new Set();
	const filesByKey: Map<string, string> = new Map();

	const options = {
		// TODO: remove this when experiments.lazyCompilation is removed
		...compiler.options.experiments.lazyCompilation,
		...compiler.options.lazyCompilation
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
	activeModules: Set<string>,
	filesByKey: Map<string, string>
) {
	const plugin = new BuiltinLazyCompilationPlugin(
		({ module, path }) => {
			const data = `${encodeURIComponent(
				module.replace(/\\/g, "/").replace(/@/g, "_")
			)}`;
			filesByKey.set(data, path);
			const active = activeModules.has(data);

			return {
				// port in server url can change frequently,
				// even configuration is totally the same
				client: `${options.client || getDefaultClient(compiler)}?${encodeURIComponent(getFullServerUrl(options))}`,
				data,
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
	activeModules: Set<string>,
	filesByKey: Map<string, string>,
	lazyCompilationPrefix: string
): MiddlewareHandler => {
	const logger = compiler.getInfrastructureLogger("LazyCompilation");

	return (req: IncomingMessage, res: ServerResponse, next?: () => void) => {
		if (!req.url?.startsWith(lazyCompilationPrefix)) {
			// only handle requests that are come from lazyCompilation
			return next?.();
		}

		const modules = req.url.slice(lazyCompilationPrefix.length).split("@");
		req.socket.setNoDelay(true);

		res.setHeader("content-type", "text/event-stream");
		res.writeHead(200);
		res.write("\n");

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
				compiler.watching.invalidateWithChangesAndRemovals(rebuiltModules);
			}
		}
	};
};
