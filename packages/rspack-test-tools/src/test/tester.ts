import {
	ITestContext,
	ITestEnv,
	ITester,
	ITesterConfig,
	ITestProcessor
} from "../type";
import { TestContext } from "./context";

export class Tester implements ITester {
	private context: ITestContext;
	private steps: ITestProcessor[] = [];
	step: number = 0;
	total: number = 0;

	constructor(private config: ITesterConfig) {
		this.context = new TestContext(config);
		this.steps = config.steps || [];
		this.step = 0;
		this.total = config.steps?.length || 0;
		if (config.contextValue) {
			for (let [key, value] of Array.from(
				Object.entries(config.contextValue)
			)) {
				this.context.setValue(config.name, key, value);
			}
		}
	}
	getContext(): ITestContext {
		return this.context;
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

		await this.runStepMethods(currentStep, [
			"before",
			"config",
			"compiler",
			"build"
		]);
	}
	async check(env: ITestEnv) {
		const currentStep = this.steps[this.step];
		if (!currentStep) return;

		await this.runCheckStepMethods(
			currentStep,
			env,
			this.context.hasError() ? ["check"] : ["run", "check"]
		);
		await this.runStepMethods(currentStep, ["after"], true);
	}

	next() {
		if (this.context.hasError()) {
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

	private async runStepMethods(
		step: ITestProcessor,
		methods: Array<"before" | "config" | "compiler" | "build" | "after">,
		force: boolean = false
	) {
		for (let i of methods) {
			if (!force && this.context.hasError()) return;
			if (typeof step[i] === "function") {
				try {
					await step[i]!(this.context);
				} catch (e) {
					this.context.emitError(this.config.name, e as Error);
				}
			}
		}
	}

	private async runCheckStepMethods(
		step: ITestProcessor,
		env: ITestEnv,
		methods: Array<"run" | "check">
	) {
		for (let i of methods) {
			if (typeof step[i] === "function") {
				await step[i]!(env, this.context);
			}
		}
	}
}
