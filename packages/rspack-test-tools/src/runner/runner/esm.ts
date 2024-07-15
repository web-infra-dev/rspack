import path from "path";
import { fileURLToPath, pathToFileURL } from "url";
import vm, { SourceTextModule } from "vm";

import asModule from "../../helper/legacy/asModule";
import type { ECompilerType } from "../../type";
import { EEsmMode, type TRunnerRequirer } from "../type";
import { CommonJsRunner } from "./cjs";

export class EsmRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends CommonJsRunner<T> {
	protected createRunner() {
		super.createRunner();
		this.requirers.set("cjs", this.getRequire());
		this.requirers.set("esm", this.createEsmRequirer());
		this.requirers.set("entry", (currentDirectory, modulePath, context) => {
			const file = this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (
				file.path.endsWith(".mjs") &&
				this._options.compilerOptions.experiments?.outputModule
			) {
				return this.requirers.get("esm")!(currentDirectory, modulePath, {
					...context,
					file
				});
			} else {
				return this.requirers.get("cjs")!(currentDirectory, modulePath, {
					...context,
					file
				});
			}
		});
	}

	protected createEsmRequirer(): TRunnerRequirer {
		const esmContext = vm.createContext(this.baseModuleScope!, {
			name: "context for esm"
		});
		const esmCache = new Map<string, SourceTextModule>();
		const esmIdentifier = this._options.name;
		return (currentDirectory, modulePath, context = {}) => {
			if (!SourceTextModule) {
				throw new Error(
					"Running this test requires '--experimental-vm-modules'.\nRun with 'node --experimental-vm-modules node_modules/jest-cli/bin/jest'."
				);
			}
			const _require = this.getRequire();
			const file =
				context["file"] || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			let esm = esmCache.get(file.path);
			if (!esm) {
				esm = new SourceTextModule(file.content, {
					identifier: esmIdentifier + "-" + file.path,
					// no attribute
					url: pathToFileURL(file.path).href + "?" + esmIdentifier,
					context: esmContext,
					initializeImportMeta: (meta: { url: string }, _: any) => {
						meta.url = pathToFileURL(file!.path).href;
					},
					importModuleDynamically: async (
						specifier: any,
						module: { context: any }
					) => {
						const result = await _require(path.dirname(file!.path), specifier, {
							esmMode: EEsmMode.Evaluated
						});
						return await asModule(result, module.context);
					}
				} as any);
				esmCache.set(file.path, esm);
			}
			if (context["esmMode"] === EEsmMode.Unlinked) return esm;
			return (async () => {
				await esm.link(async (specifier, referencingModule) => {
					return await asModule(
						await _require(
							path.dirname(
								referencingModule.identifier
									? referencingModule.identifier.slice(esmIdentifier.length + 1)
									: fileURLToPath((referencingModule as any).url)
							),
							specifier,
							{
								esmMode: EEsmMode.Unlinked
							}
						),
						referencingModule.context,
						true
					);
				});
				if ((esm as any).instantiate) (esm as any).instantiate();
				await esm.evaluate();
				if (context["esmMode"] === EEsmMode.Evaluated) {
					return esm;
				} else {
					const ns = esm.namespace as {
						default: unknown;
					};
					return ns.default && ns.default instanceof Promise ? ns.default : ns;
				}
			})();
		};
	}
}
