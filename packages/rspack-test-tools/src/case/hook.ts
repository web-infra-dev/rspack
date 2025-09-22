import path from "node:path";
import { Compilation, Compiler, sources } from "@rspack/core";
import { getSerializers } from "jest-snapshot";
import { createSnapshotSerializer as createPathSerializer } from "path-serializer";
import {
	type PrettyFormatOptions,
	format as prettyFormat
} from "pretty-format";
import merge from "webpack-merge";
import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import {
	type ISnapshotProcessorOptions,
	SnapshotProcessor
} from "../processor";
import { BasicRunnerFactory } from "../runner";
import { TestContext, type TTestContextOptions } from "../test/context";
import { getSimpleProcessorRunner } from "../test/simple";
import type { ITestEnv } from "../type";
import {
	ECompilerType,
	type ITestContext,
	type TCompiler,
	type TCompilerOptions
} from "../type";

const srcDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/fixtures"
);
const distDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/js/hook"
);

const sourceSerializer = {
	test(val: unknown) {
		return val instanceof sources.Source;
	},
	print(val: sources.Source) {
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

const testPathSerializer = createPathSerializer({
	replace: [
		{
			match: srcDir,
			mark: "<HOOK_SRC_DIR>"
		},
		{
			match: distDir,
			mark: "<HOOK_DIST_DIR>"
		}
	]
});

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
	protected count = 0;
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
		const context = this;
		return function SNAPPED_HOOK(this: any, ...args: unknown[]) {
			const group = prefix ? prefix : context.count++;
			context._addSnapshot(args, "input", group);
			const output = cb.apply(this, args);
			if (output && typeof output.then === "function") {
				let resolve: ((value: void | PromiseLike<void>) => void) | undefined;
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
		const normalizedContent = Buffer.isBuffer(content)
			? content
			: serialize(content, undefined, {
					escapeString: true,
					printBasicPrototype: true
				}).replace(/\r\n/g, "\n");

		(this.snapshots[group] = this.snapshots[group] || []).push([
			normalizedContent,
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

		const snapshots = this.snapshotsList.reduce((acc, group, index) => {
			const block = this.snapshots[group || index].reduce(
				(acc, [content, name]) => {
					name = `## ${name || `test: ${index}`}\n\n`;
					const block = `\`\`\`javascript\n${content}\n\`\`\`\n`;
					return `${acc}${name + block}\n`;
				},
				""
			);
			return `${acc}# ${Number.isInteger(group) ? `Group: ${index}` : group}\n\n${block}`;
		}, "");
		env
			.expect(snapshots)
			.toMatchFileSnapshot(path.join(this.src, "hooks.snap.txt"), options);
	}
}

export type THookCaseConfig = Omit<
	ISnapshotProcessorOptions<ECompilerType.Rspack>,
	"name" | "compilerType" | "runable"
> & {
	options?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>;
	compiler?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
	check?: (context: ITestContext) => Promise<void>;
	description: string;
};

export function createHookCase(
	name: string,
	src: string,
	dist: string,
	source: string
) {
	const caseConfig: Partial<THookCaseConfig> = require(
		path.join(src, "test.js")
	);
	const testName = path.basename(
		name.slice(0, name.indexOf(path.extname(name)))
	);
	const runner = getSimpleProcessorRunner(source, dist, {
		env: () => env,
		context: () =>
			new HookCasesContext(src, testName, {
				src: source,
				dist: dist,
				runnerFactory: BasicRunnerFactory
			})
	});

	it(caseConfig.description!, async () => {
		await runner(
			name,
			new SnapshotProcessor({
				name,
				compilerType: ECompilerType.Rspack,
				findBundle: () => ["main.js"],
				snapshot: path.join(src, "output.snap.txt"),
				runable: false,
				defaultOptions: context => {
					let defaultOptions = {
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
						},
						experiments: {
							css: true,
							rspackFuture: {
								bundlerInfo: {
									force: false
								}
							},
							inlineConst: true,
							lazyBarrel: true
						}
					} as TCompilerOptions<ECompilerType.Rspack>;
					if (caseConfig.options) {
						defaultOptions = merge(defaultOptions, caseConfig.options(context));
					}
					return defaultOptions;
				},
				overrideOptions: (context, options) => {
					if (!global.printLogger) {
						options.infrastructureLogging = {
							level: "error"
						};
					}
				},
				compiler: caseConfig.compiler,
				check: async function (env, context, compiler, stats) {
					await (context as HookCasesContext).collectSnapshots(env);
					await SnapshotProcessor.prototype.check.call(this, env, context);
					if (typeof caseConfig.check === "function") {
						await caseConfig.check(context);
					}
				},
				snapshotFileFilter: caseConfig.snapshotFileFilter
			})
		);
	});
	const env = createLazyTestEnv(10000);
}
