import type { NormalizedJsModule } from "@rspack/core";
import { Compiler, MultiCompiler } from "@rspack/core";
import { ProgressPlugin as WebpackProgressPlugin } from "webpack";

const NAME = "ProgressPlugin";

type HandlerFunction = (
	percentage: number,
	msg: string,
	...args: string[]
) => void;

type ProgressPluginArgument =
	| HandlerFunction
	| {
			handler?: HandlerFunction;
			profile?: boolean;
	  };

class ProgressPlugin {
	handler: HandlerFunction | undefined;
	profile: boolean | undefined;

	constructor(options: ProgressPluginArgument = {}) {
		if (typeof options === "function") {
			options = {
				handler: options
			};
		}

		this.profile = options.profile;
		this.handler = options.handler;
	}

	apply(compiler: Compiler) {
		if (compiler.options.builtins && compiler.options.builtins.progress) {
			return;
		}

		const handler =
			this.handler ||
			WebpackProgressPlugin.createDefaultHandler(
				this.profile,
				compiler.getInfrastructureLogger("rspack.Progress")
			);

		if (compiler instanceof MultiCompiler) {
			this._applyOnMultiCompiler(compiler, handler);
		} else if (compiler instanceof Compiler) {
			this._applyOnCompiler(compiler, handler);
		}
	}

	private _applyOnCompiler(compiler: Compiler, handler: Function) {
		let lastActiveModule = "";
		let lastModulesCount = 0;
		let lastDependenciesCount = 0;
		let lastEntriesCount = 0;
		let modulesCount = 0;
		let dependenciesCount = 0;
		let entriesCount = 1;
		let doneModules = 0;
		let doneDependencies = 0;
		let doneEntries = 0;
		const activeModules = new Set();
		let lastUpdate = 0;

		// const updateThrottled = () => {
		// 	if (lastUpdate + 500 < Date.now()) {
		// 		update();
		// 	}
		// };

		const update = () => {
			const percentByModules =
				doneModules / Math.max(lastModulesCount || 1, modulesCount);

			const percentage = 0.1 + percentByModules * 0.55;

			handler(percentage, "building", [lastActiveModule]);
			lastUpdate = Date.now();
		};

		const moduleBuild = (module: NormalizedJsModule) => {
			const ident = module.identifier();
			if (ident) {
				lastActiveModule = ident;
				activeModules.add(ident);
				update();
			}
		};

		compiler.hooks.make.tap(NAME, () => {
			handler(0.1, "building");
		});

		compiler.hooks.compilation.tap(NAME, compilation => {
			if (compilation.compiler.isChild()) {
				return;
			}
			lastModulesCount = modulesCount;
			lastEntriesCount = entriesCount;
			lastDependenciesCount = dependenciesCount;
			modulesCount = dependenciesCount = entriesCount = 0;
			doneModules = doneDependencies = doneEntries = 0;

			compilation.hooks.buildModule.tap(NAME, moduleBuild);

			compilation.hooks.optimizeChunkModules.tap(NAME, () => {
				handler(0.8, "optimizing chunks");
				return undefined;
			});

			compilation.hooks.processAssets.tap(NAME, () => {
				handler(0.9, "process assets");
			});
		});

		compiler.hooks.done.tap(NAME, () => {
			handler(1, "done");
		});
	}

	private _applyOnMultiCompiler(compiler: MultiCompiler, handler: Function) {
		const states = compiler.compilers.map(() => [0] as [number, ...string[]]);
		compiler.compilers.forEach((compiler, idx) => {
			new ProgressPlugin((p, msg, ...args) => {
				states[idx] = [p, msg, ...args];
				let sum = 0;
				for (const [p] of states) sum += p;
				handler(sum / states.length, `[${idx}] ${msg}`, ...args);
			}).apply(compiler);
		});
	}
}

export default ProgressPlugin;
