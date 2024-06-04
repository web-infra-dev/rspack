import { Compilation, Compiler } from "@rspack/core";
import { getSerializers } from "jest-snapshot";
import path from "path";
import { format as prettyFormat, PrettyFormatOptions } from "pretty-format";
import { Source } from "webpack-sources";

import { TestContext, TTestContextOptions } from "../test/context";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompiler,
	TCompilerOptions
} from "../type";
import { ISnapshotProcessorOptions, SnapshotProcessor } from "./snapshot";

const pathSerializer = require("jest-serializer-path");
const normalizePaths = pathSerializer.normalizePaths;
const srcDir = path.resolve(__dirname, "../../tests/fixtures");
const distDir = path.resolve(__dirname, "../../tests/js/hook");

const sourceSerializer = {
	test(val: unknown) {
		return val instanceof Source;
	},
	print(val: Source) {
		return val.source();
	}
};

const internalSerializer = {
	test(val: unknown) {
		return val instanceof Compiler || val instanceof Compilation;
	},
	print(val: Compiler | Compilation) {
		return JSON.stringify(`${val.constructor.name}(internal ignored)`);
	}
};

const testPathSerializer = {
	test(val: unknown) {
		return typeof val === "string";
	},
	print(val: string) {
		return JSON.stringify(
			normalizePaths(
				// @ts-ignore
				val
					.split(srcDir)
					.join("<HOOK_SRC_DIR>")
					.split(distDir)
					.join("<HOOK_DIST_DIR>")
			)
		);
	}
};

const escapeRegex = true;
const printFunctionName = false;
const normalizeNewlines = (str: string) => str.replace(/\r\n|\r/g, "\n");
const serialize = (val: unknown, indent = 2, formatOverrides = {}) =>
	normalizeNewlines(
		prettyFormat(val, {
			escapeRegex,
			indent,
			plugins: [
				...getSerializers(),
				sourceSerializer,
				internalSerializer,
				testPathSerializer
			] as PrettyFormatOptions["plugins"],
			printFunctionName,
			...formatOverrides
		})
	);

export class HookCasesContext extends TestContext {
	protected promises: Promise<void>[] = [];
	protected count: number = 0;
	protected snapshots: Record<
		string | number,
		Array<[string | Buffer, string]>
	> = {};
	protected snapshotsList: Array<string | number> = [];

	constructor(
		protected src: string,
		protected testName: string,
		protected options: TTestContextOptions
	) {
		super(options);
		this.snapped = this.snapped.bind(this);
	}

	/**
	 * Snapshot function arguments and return value.
	 * Generated snapshot is located in the same directory with the test source.
	 * @example
	 * compiler.hooks.compilation("name", context.snapped((...args) => { ... }))
	 */
	snapped(cb: (...args: unknown[]) => Promise<unknown>, prefix = "") {
		// eslint-disable-next-line
		let context = this;
		return function SNAPPED_HOOK(this: any, ...args: unknown[]) {
			let group = prefix ? prefix : context.count++;
			context._addSnapshot(args, "input", group);
			let output = cb.apply(this, args);
			if (output && typeof output.then === "function") {
				let resolve;
				context.promises.push(new Promise(r => (resolve = r)));
				return output
					.then((o: unknown) => {
						context._addSnapshot(o, "output (Promise resolved)", group);
						return o;
					})
					.catch((o: unknown) => {
						context._addSnapshot(o, "output (Promise rejected)", group);
						return o;
					})
					.finally(resolve);
			}
			context._addSnapshot(output, "output", group);
			return output;
		};
	}

	/**
	 * @internal
	 */
	_addSnapshot(content: unknown, name: string, group: string | number) {
		content = Buffer.isBuffer(content)
			? content
			: serialize(content, undefined, {
					escapeString: true,
					printBasicPrototype: true
				}).replace(/\r\n/g, "\n");
		(this.snapshots[group] = this.snapshots[group] || []).push([
			content as Buffer | string,
			name
		]);
		if (!this.snapshotsList.includes(group)) {
			this.snapshotsList.push(group);
		}
	}

	/**
	 * @internal
	 */
	async collectSnapshots(
		env: ITestEnv,
		options = {
			diff: {}
		}
	) {
		await Promise.allSettled(this.promises);
		if (!this.snapshotsList.length) return;

		let snapshots = this.snapshotsList.reduce((acc, group, index) => {
			let block = this.snapshots[group || index].reduce(
				(acc, [content, name]) => {
					name = `## ${name || `test: ${index}`}\n\n`;
					let block = "```javascript\n" + content + "\n```\n";
					return (acc += name + block + "\n");
				},
				""
			);
			group = Number.isInteger(group) ? `Group: ${index}` : group;
			group = `# ${group}\n\n`;
			return (acc += group + block);
		}, "");
		env
			.expect(snapshots)
			.toMatchFileSnapshot(path.join(this.src, "hooks.snap.txt"), options);
	}
}

export interface IHookProcessorOptions<T extends ECompilerType>
	extends ISnapshotProcessorOptions<T> {
	options?: (context: ITestContext) => TCompilerOptions<T>;
	compiler?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	check?: (context: ITestContext) => Promise<void>;
}

export class HookTaskProcessor<
	T extends ECompilerType
> extends SnapshotProcessor<T> {
	constructor(protected _hookOptions: IHookProcessorOptions<T>) {
		super({
			defaultOptions: HookTaskProcessor.defaultOptions<T>,
			..._hookOptions
		});
	}

	async config(context: ITestContext): Promise<void> {
		await super.config(context);
		const compiler = this.getCompiler(context);
		if (typeof this._hookOptions.options === "function") {
			compiler.mergeOptions(this._hookOptions.options(context));
		}
	}

	async compiler(context: ITestContext): Promise<void> {
		await super.compiler(context);
		if (typeof this._hookOptions.compiler === "function") {
			const compiler = this.getCompiler(context);
			await this._hookOptions.compiler(context, compiler.getCompiler()!);
		}
	}

	async check(env: ITestEnv, context: HookCasesContext) {
		await (context as any).collectSnapshots(env);
		await super.check(env, context);
		if (typeof this._hookOptions.check === "function") {
			await this._hookOptions.check(context);
		}
	}

	static defaultOptions<T extends ECompilerType>(
		context: ITestContext
	): TCompilerOptions<T> {
		return {
			context: context.getSource(),
			mode: "production",
			target: "async-node",
			devtool: false,
			cache: false,
			entry: "./hook",
			output: {
				path: context.getDist()
			},
			optimization: {
				minimize: false
			}
		} as TCompilerOptions<T>;
	}
}
