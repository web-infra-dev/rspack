import { ECompilerType } from "../type";
import { BasicRunner } from "./basic";
import {
	IBasicModuleScope,
	IBasicRunnerOptions,
	TBasicRunnerFile,
	TRunnerRequirer
} from "./type";
import FakeDocument from "../helper/legacy/FakeDocument";
import CurrentScript from "../helper/legacy/currentScript";
import createFakeWorker from "../helper/legacy/createFakeWorker";

export class WebRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
	private document: FakeDocument | null = null;
	private oldCurrentScript: CurrentScript | null = null;
	constructor(options: IBasicRunnerOptions<T>) {
		super({
			...options,
			runInNewContext: true
		});
	}
	run(file: string) {
		this.document = new FakeDocument(this.options.dist);
		return super.run(file);
	}
	protected createGlobalContext() {
		const globalContext = super.createGlobalContext();
		globalContext["document"] = this.document;
		globalContext["getComputedStyle"] = this.document!.getComputedStyle.bind(
			this.document
		);
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
			this.requirers.get("entry")!(
				this.options.dist,
				`.${url.slice("https://test.cases/path".length)}`
			);
		};
		return subModuleScope;
	}

	protected createBaseModuleScope() {
		const moduleScope = super.createBaseModuleScope();
		moduleScope["window"] = this.globalContext;
		moduleScope["self"] = this.globalContext;
		moduleScope["document"] = this.document;
		moduleScope["setTimeout"] = this.globalContext!.setTimeout;
		moduleScope["clearTimeout"] = this.globalContext!.clearTimeout;
		moduleScope["URL"] = URL;
		moduleScope["Worker"] = createFakeWorker({
			outputDirectory: this.options.dist
		});
		return moduleScope;
	}

	protected preExecute(_: string, file: TBasicRunnerFile): void {
		this.oldCurrentScript = this.document!.currentScript;
		this.document!.currentScript = new CurrentScript(file.subPath);
	}

	protected postExecute(_: Object, file: TBasicRunnerFile): void {
		this.document!.currentScript = this.oldCurrentScript;
		this.oldCurrentScript = null;
	}
}
