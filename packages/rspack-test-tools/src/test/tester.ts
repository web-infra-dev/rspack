import { TestContext } from "./context";
import {
	ITester,
	ITesterConfig,
	ITestContext,
	ITestProcessor,
	ITestEnv
} from "../type";

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

		try {
			await this.runStepMethods(currentStep, [
				"before",
				"config",
				"compiler",
				"build"
			]);
		} catch (e) {}
	}
	async check(env: ITestEnv) {
		const currentStep = this.steps[this.step];
		if (!currentStep) return;

		if (this.context.hasError()) {
			await this.runCheckStepMethods(currentStep, env, ["check"]);
		} else {
			await this.runCheckStepMethods(currentStep, env, ["run", "check"]);
		}
		await this.runStepMethods(currentStep, ["after"], true);

		if (this.context.hasError()) {
			this.outputErrors();
			throw new Error(`Case "${this.config.name}" check failed`);
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

	private outputErrors() {
		console.error(
			`Case "${this.config.name}" run failed: errors occur in step ${
				this.step + 1
			}:`
		);
		for (let key of this.context.getNames()) {
			const errors = this.context.getError(key);
			if (errors.length === 0) continue;
			console.error(`Error index: ${key}:`);
			for (let error of errors) {
				console.error(error);
			}
		}
	}
}
