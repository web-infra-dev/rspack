import {
	type ECompilerType,
	EDocumentType,
	type ITestRunner
} from "../../../type";
import type { INodeRunnerOptions, NodeRunner } from "../node";
import { FakeDocumentWebRunner } from "./fake";
import { JSDOMWebRunner } from "./jsdom";

export interface IWebRunnerOptions<
	T extends ECompilerType = ECompilerType.Rspack
> extends INodeRunnerOptions<T> {
	dom: EDocumentType;
}

export class WebRunner<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestRunner
{
	protected originMethods: Partial<NodeRunner<T>> = {};
	private implement: NodeRunner<T>;
	constructor(protected _webOptions: IWebRunnerOptions<T>) {
		const { dom } = _webOptions;
		if (dom === EDocumentType.Fake) {
			this.implement = new FakeDocumentWebRunner(_webOptions);
		} else if (dom === EDocumentType.JSDOM) {
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
