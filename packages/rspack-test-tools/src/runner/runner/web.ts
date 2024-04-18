import { ECompilerType } from "../../type";
import { BasicRunner } from "./basic";
import {
	IBasicRunnerOptions,
	TBasicRunnerFile,
	TRunnerRequirer
} from "../type";
import FakeDocument, { FakeElement } from "../../helper/legacy/FakeDocument";
import CurrentScript from "../../helper/legacy/currentScript";
import createFakeWorker from "../../helper/legacy/createFakeWorker";
import path from "path";
import fs from "fs";
import urlToRelativePath from "../../helper/legacy/urlToRelativePath";
import EventSource from "../../helper/legacy/EventSourceForNode";

export class WebRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
	private document: FakeDocument;
	private oldCurrentScript: CurrentScript | null = null;
	constructor(protected _webOptions: IBasicRunnerOptions<T>) {
		super({
			..._webOptions,
			runInNewContext: true
		});
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
		if (!file.endsWith(".js")) {
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
		globalContext["document"] = this.document;
		globalContext["getComputedStyle"] = this.document.getComputedStyle.bind(
			this.document
		);
		const urlToPath = (url: string) => {
			if (url.startsWith("https://test.cases/path/")) url = url.slice(24);
			return path.resolve(this._options.dist, `./${url}`);
		};

		globalContext["fetch"] = async (url: string) => {
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
		globalContext["importScripts"] = (url: string) => {
			expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
			this.requirers.get("entry")!(this._options.dist, urlToRelativePath(url));
		};
		globalContext["document"] = this.document;
		globalContext["Worker"] = createFakeWorker({
			outputDirectory: this._options.dist
		});
		globalContext["EventSource"] = EventSource;
		globalContext["location"] = {
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
		subModuleScope["importScripts"] = (url: string) => {
			expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
			this.getRequire()(
				this._options.dist,
				`.${url.slice("https://test.cases/path".length)}`
			);
		};
		return subModuleScope;
	}

	protected createBaseModuleScope() {
		const moduleScope = super.createBaseModuleScope();
		moduleScope["window"] = this.globalContext;
		moduleScope["self"] = this.globalContext;
		moduleScope["document"] = this.globalContext!["document"];
		moduleScope["fetch"] = this.globalContext!["fetch"];
		moduleScope["importScripts"] = this.globalContext!["importScripts"];
		moduleScope["Worker"] = this.globalContext!["Worker"];
		moduleScope["EventSource"] = this.globalContext!["EventSource"];
		moduleScope["URL"] = URL;
		moduleScope["Worker"] = createFakeWorker({
			outputDirectory: this._options.dist
		});
		moduleScope["__dirname"] = this._options.dist;
		return moduleScope;
	}

	protected createJsonRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			if (Array.isArray(modulePath)) {
				throw new Error("Array module path is not supported in hot cases");
			}
			let file = context["file"] || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}
			return JSON.parse(
				fs.readFileSync(path.join(this._options.dist, modulePath), "utf-8")
			);
		};
	}

	protected createRunner() {
		super.createRunner();
		this.requirers.set("cjs", this.getRequire());
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
			} else {
				return this.requirers.get("cjs")!(
					this._options.dist,
					modulePath,
					context
				);
			}
		});
	}

	protected preExecute(_: string, file: TBasicRunnerFile): void {
		this.oldCurrentScript = this.document.currentScript;
		this.document.currentScript = new CurrentScript(file.subPath);
	}

	protected postExecute(_: Object, file: TBasicRunnerFile): void {
		this.document.currentScript = this.oldCurrentScript;
		this.oldCurrentScript = null;
	}
}
