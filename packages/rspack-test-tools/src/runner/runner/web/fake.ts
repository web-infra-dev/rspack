import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import vm, { SourceTextModule } from "node:vm";
import { isCss } from "../../../helper";
import asModule from "../../../helper/legacy/asModule";
import createFakeWorker from "../../../helper/legacy/createFakeWorker";
import CurrentScript from "../../../helper/legacy/currentScript";
import EventSource from "../../../helper/legacy/EventSourceForNode";
import FakeDocument, {
	type FakeElement
} from "../../../helper/legacy/FakeDocument";
import urlToRelativePath from "../../../helper/legacy/urlToRelativePath";
import type { ECompilerType } from "../../../type";
import {
	EEsmMode,
	type TBasicRunnerFile,
	type TRunnerRequirer
} from "../../type";
import type { IBasicRunnerOptions } from "../basic";
import { CommonJsRunner } from "../cjs";

export class FakeDocumentWebRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends CommonJsRunner<T> {
	private document: FakeDocument;
	private oldCurrentScript: CurrentScript | null = null;

	constructor(protected _webOptions: IBasicRunnerOptions<T>) {
		super(_webOptions);
		this.document = new FakeDocument(_webOptions.dist, {
			onScript: (node: FakeElement) => {
				this.getRequire.bind(this)()(
					_webOptions.dist,
					urlToRelativePath(node.src)
				);
			}
		});
	}

	run(file: string) {
		if (isCss(file)) {
			const cssElement = this.document.createElement("link");
			cssElement.href = file;
			cssElement.rel = "stylesheet";
			this.document.head.appendChild(cssElement);
			return Promise.resolve();
		}
		return super.run(file);
	}

	protected createGlobalContext() {
		const globalContext = super.createGlobalContext();
		globalContext.document = this.document;
		globalContext.getComputedStyle = this.document.getComputedStyle.bind(
			this.document
		);
		const urlToPath = (url: string) => {
			return path.resolve(
				this._options.dist,
				`./${url.startsWith("https://test.cases/path/") ? url.slice(24) : url}`
			);
		};

		globalContext.fetch = async (url: string) => {
			try {
				const buffer: Buffer = await new Promise((resolve, reject) =>
					fs.readFile(urlToPath(url), (err, b) =>
						err ? reject(err) : resolve(b)
					)
				);
				return {
					status: 200,
					ok: true,
					json: async () => JSON.parse(buffer.toString("utf-8"))
				};
			} catch (err) {
				if ((err as { code: string }).code === "ENOENT") {
					return {
						status: 404,
						ok: false
					};
				}
				throw err;
			}
		};
		globalContext.importScripts = (url: string) => {
			this._options.env.expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
			this.requirers.get("entry")!(this._options.dist, urlToRelativePath(url));
		};
		globalContext.document = this.document;
		globalContext.Worker = createFakeWorker(this._options.env, {
			outputDirectory: this._options.dist
		});
		globalContext.EventSource = EventSource;
		globalContext.location = {
			href: "https://test.cases/path/index.html",
			origin: "https://test.cases",
			toString() {
				return "https://test.cases/path/index.html";
			}
		};
		return globalContext;
	}

	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: any,
		file: TBasicRunnerFile
	) {
		const subModuleScope = super.createModuleScope(requireFn, m, file);
		subModuleScope.importScripts = (url: string) => {
			this._options.env.expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
			this.getRequire()(
				this._options.dist,
				`.${url.slice("https://test.cases/path".length)}`
			);
		};
		return subModuleScope;
	}

	protected createBaseModuleScope() {
		const moduleScope = super.createBaseModuleScope();
		moduleScope.window = this.globalContext;
		moduleScope.self = this.globalContext;
		moduleScope.globalThis = this.globalContext;
		moduleScope.document = this.globalContext!.document;

		moduleScope.getComputedStyle = this.globalContext!.getComputedStyle.bind(
			this.globalContext
		);
		moduleScope.fetch = this.globalContext!.fetch;
		moduleScope.importScripts = this.globalContext!.importScripts;
		moduleScope.Worker = this.globalContext!.Worker;
		moduleScope.EventSource = this.globalContext!.EventSource;
		moduleScope.URL = URL;
		moduleScope.Worker = createFakeWorker(this._options.env, {
			outputDirectory: this._options.dist
		});
		moduleScope.__dirname = this._options.dist;
		return moduleScope;
	}

	protected createRunner() {
		super.createRunner();
		this.requirers.set("cjs", this.getRequire());
		this.requirers.set("mjs", this.createESMRequirer());
		this.requirers.set("json", this.createJsonRequirer());

		this.requirers.set("entry", (_, modulePath, context) => {
			if (Array.isArray(modulePath)) {
				throw new Error("Array module path is not supported in web runner");
			}
			if (modulePath.endsWith(".json")) {
				return this.requirers.get("json")!(
					this._options.dist,
					modulePath,
					context
				);
			} else if (modulePath.endsWith(".mjs")) {
				return this.requirers.get("mjs")!(
					this._options.dist,
					modulePath,
					context
				);
			}
			return this.requirers.get("cjs")!(
				this._options.dist,
				modulePath,
				context
			);
		});
	}

	protected preExecute(_: string, file: TBasicRunnerFile): void {
		this.oldCurrentScript = this.document.currentScript;
		this.document.currentScript = new CurrentScript(file.subPath);
		super.preExecute(_, file);
	}

	protected postExecute(_: Object, file: TBasicRunnerFile): void {
		super.postExecute(_, file);
		this.document.currentScript = this.oldCurrentScript;
		this.oldCurrentScript = null;
	}

	private createESMRequirer(): TRunnerRequirer {
		const esmContext = vm.createContext(
			{
				...this.baseModuleScope!,
				...this.globalContext
			},
			{
				name: "context for esm"
			}
		);
		const esmCache = new Map<string, SourceTextModule>();

		return (
			currentDirectory: string,
			modulePath: string | string[],
			context: any = {}
		) => {
			const esmIdentifier = `esm-${currentDirectory}-${modulePath}`;
			if (!SourceTextModule) {
				throw new Error(
					"Running this test requires '--experimental-vm-modules'.\nRun with 'node --experimental-vm-modules node_modules/jest-cli/bin/jest'."
				);
			}
			const _require = this.getRequire();

			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			let esm = esmCache.get(file.path);
			if (!esm) {
				esm = new SourceTextModule(file.content, {
					identifier: `${esmIdentifier}-${file.path}`,
					// no attribute
					url: `${pathToFileURL(file.path).href}?${esmIdentifier}`,
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
			if (context.esmMode === EEsmMode.Unlinked) return esm;
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
				if (context.esmMode === EEsmMode.Evaluated) {
					return esm;
				}
				const ns = esm.namespace as {
					default: unknown;
				};
				return ns.default && ns.default instanceof Promise ? ns.default : ns;
			})();
		};
	}
}
