import path from "node:path";
import type { Compiler as RspackCompiler } from "@rspack/core";
import type {
	Compilation as WebpackCompilation,
	Compiler as WebpackCompiler
} from "webpack";

import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerMultiStats,
	TCompilerStats
} from "../type";
import { BasicProcessor, type IBasicProcessorOptions } from "./basic";

export interface ISnapshotProcessorOptions<T extends ECompilerType>
	extends IBasicProcessorOptions<T> {
	snapshot: string;
	snapshotFileFilter?: (file: string) => boolean;
}

export class SnapshotProcessor<
	T extends ECompilerType
> extends BasicProcessor<T> {
	constructor(protected _snapshotOptions: ISnapshotProcessorOptions<T>) {
		super(_snapshotOptions);
		if (path.extname(_snapshotOptions.snapshot) === ".snap") {
			throw new Error(
				"Snapshot with `.snap` will be managed by jest, please use `.snap.txt` instead in SnapshotProcessor"
			);
		}
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		const c = compiler.getCompiler();
		if (!stats || !c) return;

		if (stats.hasErrors()) {
			const errors = [];
			if ((stats as TCompilerMultiStats<T>).stats) {
				for (const s of (stats as TCompilerMultiStats<T>).stats) {
					if (s.hasErrors()) {
						errors.push(...s.compilation.errors);
					}
				}
			} else {
				const s = stats as TCompilerStats<T>;
				errors.push(...s.compilation.errors);
			}

			throw new Error(
				`Failed to compile in fixture ${this._options.name}, Errors: ${errors
					?.map(i => `${i.message}\n${i.stack}`)
					.join("\n\n")}`
			);
		}
		const compilation =
			(c as RspackCompiler)._lastCompilation ||
			(
				c as WebpackCompiler & {
					_lastCompilation: WebpackCompilation;
				}
			)._lastCompilation;

		const snapshotFileFilter =
			this._snapshotOptions.snapshotFileFilter ||
			((file: string) => file.endsWith(".js") && !file.includes("runtime.js"));
		const fileContents = Object.entries(compilation.assets)
			.filter(([file]) => snapshotFileFilter(file))
			.map(([file, source]) => {
				const tag = path.extname(file).slice(1) || "txt";
				const content = this.serializeEachFile(source.source().toString());

				return `\`\`\`${tag} title=${file}\n${content}\n\`\`\``;
			});
		fileContents.sort();
		const content = fileContents.join("\n\n");
		const snapshotPath = path.isAbsolute(this._snapshotOptions.snapshot)
			? this._snapshotOptions.snapshot
			: path.resolve(
					context.getSource(),
					`./__snapshots__/${this._snapshotOptions.snapshot}`
				);

		env.expect(content).toMatchFileSnapshot(snapshotPath);
	}

	serializeEachFile(content: string): string {
		return content;
	}
}
