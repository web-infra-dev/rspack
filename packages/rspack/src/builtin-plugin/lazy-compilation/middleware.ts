import type { IncomingMessage, ServerResponse } from "node:http";
import type { Compiler, LazyCompilationOptions } from "../..";
import type { Module } from "../../Module";
import { BuiltinLazyCompilationPlugin } from "./lazyCompilation";

export const LAZY_COMPILATION_PREFIX = "/lazy-compilation-using-";

const getDefaultClient = (compiler: Compiler): string =>
	require.resolve(
		`../hot/lazy-compilation-${
			compiler.options.externalsPresets.node ? "node" : "web"
		}.js`
	);

export const lazyCompilationMiddleware = (
	compiler: Compiler,
	options: LazyCompilationOptions = {}
) => {
	const activeModules = new Map();
	const filesByKey: Map<string, string> = new Map();
	new BuiltinLazyCompilationPlugin(
		({ module, path }) => {
			const key = `${encodeURIComponent(
				module.replace(/\\/g, "/").replace(/@/g, "_")
			).replace(/%(2F|3A|24|26|2B|2C|3B|3D|3A)/g, decodeURIComponent)}`;
			filesByKey.set(key, path);
			const active = activeModules.get(key) > 0;
			return {
				client: `${options.backend?.client || getDefaultClient(compiler)}?${encodeURIComponent((options.backend?.host ?? "") + LAZY_COMPILATION_PREFIX)}`,
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

	return lazyCompilationMiddlewareInternal(compiler, activeModules, filesByKey);
};

// used for reuse code, do not export this
const lazyCompilationMiddlewareInternal = (
	compiler: Compiler,
	activeModules: Map<string, boolean>,
	filesByKey: Map<string, string>
) => {
	const logger = compiler.getInfrastructureLogger("LazyCompilation");

	return (req: IncomingMessage, res: ServerResponse, next?: () => void) => {
		if (!req.url?.startsWith(LAZY_COMPILATION_PREFIX)) {
			// only handle requests that are come from lazyCompilation
			return next?.();
		}

		const keys = req.url.slice(LAZY_COMPILATION_PREFIX.length).split("@");
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
							logger.warn(
								`Cannot find correct file path for module ${{ key }}`
							);
						}
						return filePath;
					})
					.filter(Boolean) as string[]
			);

			if (rebuiltModules.size) {
				compiler.watching.lazyCompilationInvalidate(new Set(rebuiltModules));
			}
		}
	};
};
