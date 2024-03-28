import { BasicTaskProcessor, IBasicProcessorOptions } from "./basic";
import { ECompilerType, ITestContext, ITestEnv } from "../type";
import path from "path";
import fs from "fs-extra";
import { type Compiler as RspackCompiler } from "@rspack/core";
import {
	type Compilation as WebpackCompilation,
	type Compiler as WebpackCompiler
} from "webpack";

declare var global: {
	updateSnapshot: boolean;
};

export interface ISnapshotProcessorOptions<T extends ECompilerType>
	extends IBasicProcessorOptions<T> {
	snapshot: string;
}

export class SnapshotProcessor<
	T extends ECompilerType
> extends BasicTaskProcessor<T> {
	constructor(protected _snapshotOptions: ISnapshotProcessorOptions<T>) {
		super(_snapshotOptions);
		if (path.extname(_snapshotOptions.snapshot) === ".snap") {
			throw new Error(
				"Snapshot with `.snap` will be managed by jest, please use `.txt` instead in SnapshotProcessor"
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
				`Failed to compile in fixture ${this._options.name}, Errors: ${
					stats.toJson({ errors: true, all: false }).errors
				}`
			);
		}
		const compilation =
			(c as RspackCompiler).compilation ||
			(
				c as WebpackCompiler & {
					_lastCompilation: WebpackCompilation;
				}
			)._lastCompilation;
		const fileContents = Object.entries(compilation.assets)
			.filter(([file]) => file.endsWith(".js") && !file.includes("runtime.js"))
			.map(([file, source]) => {
				const tag = path.extname(file).slice(1) || "txt";
				return `\`\`\`${tag} title=${file}\n${source
					.source()
					.toString()}\n\`\`\``;
			});
		fileContents.sort();
		const content = fileContents.join("\n\n");
		const snapshotPath = path.resolve(
			context.getSource(),
			`./snapshot/${this._snapshotOptions.snapshot}`
		);

		if (!fs.existsSync(snapshotPath) || global.updateSnapshot) {
			fs.writeFileSync(snapshotPath, content, "utf-8");
			return;
		}
		const snapshotContent = fs.readFileSync(snapshotPath, "utf-8");
		expect(content.trim()).toBe(snapshotContent.trim());
	}
}
