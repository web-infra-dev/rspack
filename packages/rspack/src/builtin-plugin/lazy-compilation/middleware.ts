import type { IncomingMessage, ServerResponse } from "node:http";
import { type Compiler, type LazyCompilationOptions } from "../..";
import { Module } from "../../Module";
import { BuiltinLazyCompilationPlugin } from "./lazyCompilation";
import { BuiltinPluginName } from "@rspack/binding";

export const LAZY_COMPILATION_PREFIX = "/lazy-compilation-using-";

const getDefaultClient = (compiler: Compiler): string =>
	require.resolve(
		`../hot/lazy-compilation-${
			compiler.options.externalsPresets.node ? "node" : "web"
		}.js`
	);

export const lazyCompilationMiddleware = (
	compiler: Compiler,
	options: LazyCompilationOptions = {},
	attachPlugin: boolean = true
) => {
	const activeModules = new Map();
	const filesByKey: Map<string, string> = new Map();
	const logger = compiler.getInfrastructureLogger("LazyCompilation");

	if (attachPlugin) {
		if (
			compiler.__internal__builtinPlugins.find(plugin => {
				return plugin.name === BuiltinPluginName.LazyCompilationPlugin;
			})
		) {
			logger.warn(
				`lazyCompilationMiddleware will apply ${BuiltinPluginName.LazyCompilationPlugin} by default, but you seems have already applied ${BuiltinPluginName.LazyCompilationPlugin}, you can pass the third parameter as false to disable middleware applied plugin again.`
			);
		}

		new BuiltinLazyCompilationPlugin(
			({ module, path }) => {
				const key = `${encodeURIComponent(
					module.replace(/\\/g, "/").replace(/@/g, "_")
				).replace(/%(2F|3A|24|26|2B|2C|3B|3D|3A)/g, decodeURIComponent)}`;
				filesByKey.set(key, path);
				const active = activeModules.get(key) > 0;
				return {
					client: `${options.backend?.client || getDefaultClient(compiler)}?${encodeURIComponent(LAZY_COMPILATION_PREFIX)}`,
					data: key,
					active
				};
			},
			true,
			options.entries || true,
			options.imports || true,
			typeof options.test === "function"
				? js_module => {
						const test = options.test as (module: Module) => boolean;
						return test(Module.__from_binding(js_module));
					}
				: options.test
		).apply(compiler);
	}

	return (req: IncomingMessage, res: ServerResponse, next?: () => void) => {
		if (!req.url?.startsWith(LAZY_COMPILATION_PREFIX)) {
			// only handle requests that are come from lazyCompilation
			return next?.();
		}

		const keys = req.url.slice(LAZY_COMPILATION_PREFIX.length).split("@");
		req.socket.setNoDelay(true);
		res.setHeaders(
			new Headers({
				"content-type": "text/event-stream"
			})
		);
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
						const filePath = filesByKey.get(key)!;
						if (!filePath) {
							logger.warn(
								`Cannot find correct file path for module ${{ key }}`
							);
						}
						return filePath;
					})
					.filter(Boolean)
			);

			if (rebuiltModules.size) {
				compiler.watching.lazyCompilationInvalidate(new Set(rebuiltModules));
			}
		}
	};
};
