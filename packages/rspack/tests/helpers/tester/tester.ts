import { TestContext } from "./context";
import { ITester, ITesterConfig, ITestContext, ITestProcessor } from "./type";

export class Tester implements ITester {
	private context: ITestContext;
	private steps: ITestProcessor[] = [];
	step: number = 0;

	constructor(config: ITesterConfig) {
		this.context = new TestContext(config);
		this.steps = config.steps || [];
		this.step = 0;
	}
	async prepare() {
		for (let i of this.steps) {
			if (typeof i.beforeAll === "function") {
				await i.beforeAll(this.context);
			}
		}
	}
	async compile() {
		const currentStep = this.steps[this.step];
		if (!currentStep) return;

		for (let i of ["before", "config", "compiler", "build"]) {
			if (this.context.hasError()) break;
			if (typeof currentStep[i] === "function") {
				await currentStep[i](this.context);
			}
		}

		if (this.context.hasError()) {
			this.outputErrors();
			throw new Error("build failed");
		}
	}
	async check() {
		const currentStep = this.steps[this.step];
		if (!currentStep) return;

		for (let i of ["run", "check", "after"]) {
			if (this.context.hasError()) break;
			if (typeof currentStep[i] === "function") {
				await currentStep[i](this.context);
			}
		}

		if (this.context.hasError()) {
			this.outputErrors();
			throw new Error("check failed");
		}
	}

	next() {
		if (this.context.hasError()) {
			this.outputErrors();
			return false;
		}
		if (this.steps[this.step + 1]) {
			this.step++;
			return true;
		} else {
			return false;
		}
	}

	async resume() {
		for (let i of this.steps) {
			if (typeof i.afterAll === "function") {
				await i.afterAll(this.context);
			}
		}
	}

	private outputErrors() {
		console.error(`Errors occur in step ${this.step + 1}:`);
		for (let error of this.context.errors) {
			console.error(error);
		}
	}
}
