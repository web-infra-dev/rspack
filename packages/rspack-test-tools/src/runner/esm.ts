import { ECompilerType } from "../type";
import vm, { SourceTextModule } from "vm";
import path from "path";
import { pathToFileURL } from "url";
import asModule from "../helper/legacy/asModule";
import {
	EEsmMode,
	IBasicGlobalContext,
	IBasicModuleScope,
	TRunnerRequirer
} from "./type";
import { BasicRunner } from "./basic";

export class EsmRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
	protected createEsmRequirer(
		moduleScope: IBasicModuleScope,
		globalContext: IBasicGlobalContext
	): TRunnerRequirer {
		const esmContext = vm.createContext(moduleScope, {
			name: "context for esm"
		});
		const esmCache = new Map<string, SourceTextModule>();
		const esmIdentifier = this.options.name;
		return (currentDirectory, modulePath, context = {}) => {
			if (!SourceTextModule) {
				throw new Error(
					"Running this test requires '--experimental-vm-modules'.\nRun with 'node --experimental-vm-modules node_modules/jest-cli/bin/jest'."
				);
			}
			let file = context["file"] || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			let esm = esmCache.get(file.path);
			if (!esm) {
				esm = new SourceTextModule(file.content, {
					identifier: esmIdentifier + "-" + file.path,
					// no attribute
					// url: pathToFileURL(p).href + "?" + esmIdentifier,
					context: esmContext,
					initializeImportMeta: (meta: { url: string }, _: any) => {
						meta.url = pathToFileURL(file!.path).href;
					},
					// wrong type
					importModuleDynamically: async (
						specifier: any,
						module: { context: any }
					) => {
						const result = await this.requirers.get("entry")!(
							path.dirname(file!.path),
							specifier,
							{
								esmMode: EEsmMode.Evaluated
							}
						);
						return await asModule(result, module.context);
					}
				} as any);
			}
			esmCache.set(file.path, esm);
			if (context["esmMode"] === EEsmMode.Unlinked) return esm;
			return (async () => {
				await esm.link(async (specifier, referencingModule) => {
					return await asModule(
						await this.requirers.get("entry")!(
							path.dirname(
								referencingModule.identifier.slice(esmIdentifier.length + 1)
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

	protected createRunner(
		moduleScope: IBasicModuleScope,
		globalContext: IBasicGlobalContext
	) {
		super.createRunner(moduleScope, globalContext);
		this.requirers.set("cjs", this.requirers.get("entry")!);

		this.requirers.set(
			"esm",
			this.createEsmRequirer(moduleScope, globalContext)
		);
		this.requirers.set("entry", (currentDirectory, modulePath, context) => {
			let file = this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (
				/* p.endsWith(".mjs") &&  */ this.options.compilerOptions.experiments
					?.outputModule
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
}
