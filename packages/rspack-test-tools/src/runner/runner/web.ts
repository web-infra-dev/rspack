import { ECompilerType, ITestRunner } from "../../type";
import { BasicRunner, IBasicRunnerOptions } from "./basic";
import { CommonJsRunner } from "./cjs";
import { FakeDocumentWebRunner } from "./web/fake";
import { JSDOMWebRunner } from "./web/jsdom";

export interface IWebRunnerOptions<
	T extends ECompilerType = ECompilerType.Rspack
> extends IBasicRunnerOptions<T> {
	dom: "fake" | "jsdom";
}

export class WebRunner<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestRunner
{
	protected originMethods: Partial<CommonJsRunner> = {};
	private implement: BasicRunner<T>;
	constructor(protected _webOptions: IWebRunnerOptions<T>) {
		const { dom } = _webOptions;
		if (dom === "fake") {
			this.implement = new FakeDocumentWebRunner(_webOptions);
		} else if (dom === "jsdom") {
			this.implement = new JSDOMWebRunner(_webOptions);
		} else {
			throw new Error(`Dom type "${dom}" of web runner is not support yet`);
		}
	}

	run(file: string) {
		return this.implement.run(file);
	}

	getRequire() {
		return this.implement.getRequire();
	}

	getGlobal(name: string): unknown {
		return this.implement.getGlobal(name);
	}
}
