import fs from "fs";
import { Volume, createFsFromVolume } from "memfs";

import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "../type";
import { SimpleTaskProcessor } from "./simple";
const serializer = require("jest-serializer-path");

export interface IStatsAPIProcessorOptions<T extends ECompilerType> {
	options?: (context: ITestContext) => TCompilerOptions<T>;
	name: string;
	compilerType: T;
	compiler?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	build?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	check?: (stats: TCompilerStats<T>, compiler: TCompiler<T>) => Promise<void>;
}

export class StatsAPIProcessor<
	T extends ECompilerType
> extends SimpleTaskProcessor<T> {
	constructor(protected _statsAPIOptions: IStatsAPIProcessorOptions<T>) {
		super({
			options: _statsAPIOptions.options,
			build: _statsAPIOptions.build,
			compilerType: _statsAPIOptions.compilerType,
			name: _statsAPIOptions.name,
			compiler: _statsAPIOptions.compiler
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
		// do nothing
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		env.expect(typeof stats).toBe("object");
		await this._statsAPIOptions.check?.(stats!, compiler.getCompiler()!);
	}

	static addSnapshotSerializer(expectImpl: jest.Expect) {
		expectImpl.addSnapshotSerializer(serializer);
	}
}
