import path from "node:path";
import fs from "fs-extra";
import { rimrafSync } from "rimraf";

import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import { DiffProcessor, type IDiffProcessorOptions } from "../processor";
import { Tester } from "../test/tester";
import { ECompareResultType, type TModuleCompareResult } from "../type";

export type TDiffCaseConfig = IDiffProcessorOptions;

const DEFAULT_CASE_CONFIG: Partial<IDiffProcessorOptions> = {
	webpackPath: require.resolve("webpack"),
	rspackPath: require.resolve("@rspack/core"),
	files: ["bundle.js"],
	bootstrap: true,
	detail: true,
	errors: false
};

type TFileCompareResult = {
	modules: TModuleCompareResult[];
	runtimeModules: TModuleCompareResult[];
};

export function createDiffCase(name: string, src: string, dist: string) {
	const caseConfigFile = path.join(src, "test.config.js");
	if (!fs.existsSync(caseConfigFile)) {
		return;
	}
	const caseConfig: IDiffProcessorOptions = Object.assign(
		{},
		DEFAULT_CASE_CONFIG,
		require(caseConfigFile)
	);

	const [processor, compareMap] = createDiffProcessor(caseConfig);
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [processor]
	});

	rimrafSync(dist);
	const buildTask: Promise<void> | null = tester.compile();

	const prefix = path.basename(name);
	describe(`${prefix}:check`, () => {
		beforeAll(async () => {
			await buildTask;
			compareMap.clear();
			await tester.check(env);
		});
		for (const file of caseConfig.files!) {
			describe(`Comparing "${file}"`, () => {
				let moduleResults: TModuleCompareResult[] = [];
				let runtimeResults: TModuleCompareResult[] = [];
				beforeAll(() => {
					const fileResult = compareMap.get(file);
					if (!fileResult) {
						throw new Error(`File ${file} has no results`);
					}
					moduleResults = fileResult.modules;
					runtimeResults = fileResult.runtimeModules;
				});
				if (caseConfig.modules) {
					checkCompareResults("modules", () => moduleResults);
				}
				if (caseConfig.runtimeModules) {
					checkCompareResults("runtime modules", () => runtimeResults);
				}
			});
		}
		const env = createLazyTestEnv(1000);
	});
}

function createDiffProcessor(config: IDiffProcessorOptions) {
	const fileCompareMap: Map<string, TFileCompareResult> = new Map();
	const createCompareResultHandler = (type: keyof TFileCompareResult) => {
		return (file: string, results: TModuleCompareResult[]) => {
			const fileResult = fileCompareMap.get(file) || {
				modules: [],
				runtimeModules: []
			};
			fileResult[type] = results;
			fileCompareMap.set(file, fileResult);
		};
	};

	const processor = new DiffProcessor({
		webpackPath: config.webpackPath,
		rspackPath: config.rspackPath,
		files: config.files,
		modules: config.modules,
		runtimeModules: config.runtimeModules,
		renameModule: config.renameModule,
		ignoreModuleId: config.ignoreModuleId ?? true,
		ignoreModuleArguments: config.ignoreModuleArguments ?? true,
		ignorePropertyQuotationMark: config.ignorePropertyQuotationMark ?? true,
		ignoreBlockOnlyStatement: config.ignoreBlockOnlyStatement ?? true,
		ignoreIfCertainCondition: config.ignoreIfCertainCondition ?? true,
		ignoreSwcHelpersPath: config.ignoreSwcHelpersPath ?? true,
		ignoreObjectPropertySequence: config.ignoreObjectPropertySequence ?? true,
		ignoreCssFilePath: config.ignoreCssFilePath ?? true,
		onCompareModules: createCompareResultHandler("modules"),
		onCompareRuntimeModules: createCompareResultHandler("runtimeModules"),
		bootstrap: config.bootstrap ?? true,
		detail: config.detail ?? true,
		errors: config.errors ?? false,
		replacements: config.replacements
	});

	return [processor, fileCompareMap] as [
		DiffProcessor,
		Map<string, TFileCompareResult>
	];
}

function checkCompareResults(
	name: string,
	getResults: () => TModuleCompareResult[]
) {
	describe(`Comparing ${name}`, () => {
		it("should not miss any module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.Missing)
					.map(i => i.name)
			).toEqual([]);
		});
		it("should not have any rspack-only module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.OnlySource)
					.map(i => i.name)
			).toEqual([]);
		});
		it("should not have any webpack-only module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.OnlyDist)
					.map(i => i.name)
			).toEqual([]);
		});
		it("all modules should be the same", () => {
			for (const result of getResults().filter(
				i => i.type === ECompareResultType.Different
			)) {
				console.log(`${result.name}:\n${result.detail}`);
			}
			expect(
				getResults().every(i => i.type === ECompareResultType.Same)
			).toEqual(true);
		});
	});
}
