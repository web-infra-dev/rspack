import createHotDocument from "../helper/legacy/createHotDocument";
import urlToRelativePath from "../helper/legacy/urlToRelativePath";
import createFakeWorker from "../helper/legacy/createFakeWorker";
import EventSource from "../helper/legacy/EventSourceForNode";
import { ECompilerType } from "../type";
import { BasicRunner } from "./basic";
import {
	IBasicModuleScope,
	IBasicRunnerOptions,
	TBasicRunnerFile,
	TRunnerRequirer
} from "./type";
import fs from "fs";
import path from "path";

interface IHotRunnerOptionsr<T extends ECompilerType = ECompilerType.Rspack>
	extends IBasicRunnerOptions<T> {
	next: (callback: () => {}) => void;
}

export class HotRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
	private document: any;
	constructor(protected _options: IHotRunnerOptionsr<T>) {
		super(_options);
		this.document = createHotDocument(
			_options.dist,
			this.getRequire.bind(this)
		);
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
	): IBasicModuleScope {
		const moduleScope = super.createModuleScope(requireFn, m, file);
		moduleScope["__dirname"] = this._options.dist;
		moduleScope["window"] = this.globalContext;
		moduleScope["self"] = this.globalContext;
		moduleScope["fetch"] = this.globalContext!["fetch"];
		moduleScope["document"] = this.globalContext!["document"];
		moduleScope["importScripts"] = this.globalContext!["importScripts"];
		moduleScope["Worker"] = this.globalContext!["Worker"];
		moduleScope["EventSource"] = this.globalContext!["EventSource"];
		moduleScope["STATS"] = moduleScope.__STATS__;
		moduleScope["NEXT"] = this._options.next;
		return moduleScope;
	}

	protected createRunner() {
		super.createRunner();
		this.requirers.set("cjs", this.getRequire());
		this.requirers.set("entry", (_, modulePath, context) => {
			if (Array.isArray(modulePath)) {
				throw new Error("Array module path is not supported in hot cases");
			}
			if (!modulePath.startsWith("./")) {
				return require(modulePath);
			}
			if (modulePath.endsWith(".json")) {
				return JSON.parse(
					fs.readFileSync(path.join(this._options.dist, modulePath), "utf-8")
				);
			} else {
				return this.requirers.get("cjs")!(this._options.dist, modulePath, {
					...context
				});
			}
		});
	}
}
