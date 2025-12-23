import { JsCoordinator } from "@rspack/binding";
import { type Compiler, GET_COMPILER_ID } from "../../Compiler";
import type { Compilation } from "../../exports";

const PLUGIN_NAME = "RscPlugin";

export const GET_OR_INIT_BINDING = Symbol("GET_OR_INIT_BINDING");

export class Coordinator {
	#serverCompiler?: Compiler;
	#clientCompiler?: Compiler;
	#clientLastCompilation?: Compilation;

	#binding?: JsCoordinator;

	constructor() {
		// Make the symbol method non-enumerable (and avoid TS emitting it as a class method).
		Object.defineProperty(this, GET_OR_INIT_BINDING, {
			enumerable: false,
			configurable: false,
			writable: false,
			value: () => {
				if (!this.#binding) {
					this.#binding = new JsCoordinator(() => {
						if (!this.#serverCompiler) {
							throw new Error(
								"[RscPlugin] Coordinator.getOrInitBinding() called before the server compiler was attached. " +
									"Call coordinator.applyServerCompiler(serverCompiler) first."
							);
						}
						// @ts-ignore
						return this.#serverCompiler[GET_COMPILER_ID]();
					});
				}
				return this.#binding;
			}
		});
	}

	applyServerCompiler(serverCompiler: Compiler) {
		this.#serverCompiler = serverCompiler;

		// Make server's watched dependencies include client dependencies (so server watcher stays authoritative).
		serverCompiler.hooks.done.tap(PLUGIN_NAME, stats => {
			if (this.#clientLastCompilation) {
				stats.compilation.fileDependencies.addAll(
					this.#clientLastCompilation.fileDependencies
				);
				stats.compilation.contextDependencies.addAll(
					this.#clientLastCompilation.contextDependencies
				);
				stats.compilation.missingDependencies.addAll(
					this.#clientLastCompilation.missingDependencies
				);
			}
		});

		// Server owns watch events; on invalid, explicitly invalidate client.
		serverCompiler.hooks.watchRun.tap(PLUGIN_NAME, () => {
			this.#clientCompiler!.watching!.invalidateWithChangesAndRemovals(
				new Set(this.#serverCompiler!.modifiedFiles),
				new Set(this.#serverCompiler!.removedFiles)
			);
		});
	}

	applyClientCompiler(clientCompiler: Compiler) {
		this.#clientCompiler = clientCompiler;
		const originalWatch = clientCompiler.watch;
		// Ensure client compiler watches nothing.
		// This prevents duplicate rebuilds caused by both server & client receiving FS events.
		clientCompiler.watch = function watch(watchOptions, handler) {
			watchOptions.ignored = () => true;
			return originalWatch.call(this, watchOptions, handler);
		};
		clientCompiler.hooks.done.tap(PLUGIN_NAME, stats => {
			this.#clientLastCompilation = stats.compilation;
		});
	}
}
