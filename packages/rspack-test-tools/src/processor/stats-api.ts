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
import path from "path";
const serializer = require("jest-serializer-path");
const FAKE_CWD = path.resolve(__dirname, "../../../rspack");
const CWD = process.cwd();

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

	async before(context: ITestContext): Promise<void> {
		process.chdir(this._statsAPIOptions.cwd || FAKE_CWD);
	}
	async after(context: ITestContext): Promise<void> {
		process.chdir(CWD);
	}

	static addSnapshotSerializer() {
		expect.addSnapshotSerializer(serializer);
	}
}
