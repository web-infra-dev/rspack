import { SimpleTaskProcessor } from "./simple";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "../type";
import { createFsFromVolume, Volume } from "memfs";
import fs from "fs";
const serializer = require("jest-serializer-path");

export interface IStatsAPITaskProcessorOptions<T extends ECompilerType> {
	options?: (context: ITestContext) => TCompilerOptions<T>;
	name: string;
	cwd?: string;
	compilerType: T;
	build?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	check?: (stats: TCompilerStats<T>, compiler: TCompiler<T>) => Promise<void>;
}

export class StatsAPITaskProcessor<
	T extends ECompilerType
> extends SimpleTaskProcessor<T> {
	constructor(protected _statsAPIOptions: IStatsAPITaskProcessorOptions<T>) {
		super({
			options: _statsAPIOptions.options,
			build: _statsAPIOptions.build,
			compilerType: _statsAPIOptions.compilerType,
			name: _statsAPIOptions.name
		});
	}

	async compiler(context: ITestContext) {
		await super.compiler(context);
		const compiler = this.getCompiler(context).getCompiler();
		if (compiler) {
			compiler.outputFileSystem = createFsFromVolume(
				new Volume()
			) as unknown as typeof fs;
		}
	}
	async run(env: ITestEnv, context: ITestContext) {
		throw new Error("Not support");
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		expect(typeof stats).toBe("object");
		await this._statsAPIOptions.check?.(stats!, compiler.getCompiler()!);
	}

	static addSnapshotSerializer() {
		expect.addSnapshotSerializer(serializer);
	}
}
