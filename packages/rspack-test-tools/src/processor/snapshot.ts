import { BasicTaskProcessor, IBasicProcessorOptions } from "./basic";
import { ECompilerType, ITestContext, ITestEnv } from "../type";
import path from "path";
import fs from "fs-extra";
import { type Compiler as RspackCompiler } from "@rspack/core";
import {
	type Compilation as WebpackCompilation,
	type Compiler as WebpackCompiler
} from "webpack";
import { escapeEOL } from "../helper";

declare var global: {
	updateSnapshot: boolean;
};

export interface ISnapshotProcessorOptions<T extends ECompilerType>
	extends IBasicProcessorOptions<T> {
	snapshot: string;
	snapshotFileFilter?: (file: string) => boolean;
}

export class SnapshotProcessor<
	T extends ECompilerType
> extends BasicTaskProcessor<T> {
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
			throw new Error(
				`Failed to compile in fixture ${this._options.name}, Errors: ${stats
					.toJson({ errors: true, all: false })
					.errors?.map(i => `${i.message}\n${i.stack}`)
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
				return `\`\`\`${tag} title=${file}\n${source
					.source()
					.toString()}\n\`\`\``;
			});
		fileContents.sort();
		const content = escapeEOL(fileContents.join("\n\n"));
		const snapshotPath = path.isAbsolute(this._snapshotOptions.snapshot)
			? this._snapshotOptions.snapshot
			: path.resolve(
					context.getSource(),
					`./snapshot/${this._snapshotOptions.snapshot}`
				);

		if (!fs.existsSync(snapshotPath) || global.updateSnapshot) {
			fs.ensureDirSync(path.dirname(snapshotPath));
			fs.writeFileSync(snapshotPath, content, "utf-8");
			return;
		}
		const snapshotContent = escapeEOL(fs.readFileSync(snapshotPath, "utf-8"));
		expect(content).toBe(snapshotContent);
	}
}
