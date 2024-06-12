import path from "path";
import fs from "fs-extra";
import rimraf from "rimraf";

import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import { DiffProcessor, IDiffProcessorOptions } from "../processor";
import { Tester } from "../test/tester";
import { ECompareResultType, TModuleCompareResult } from "../type";

export type TDiffCaseConfig = IDiffProcessorOptions;

const DEFAULT_CASE_CONFIG: Partial<IDiffProcessorOptions> = {
	webpackPath: require.resolve("webpack"),
	rspackPath: require.resolve("@rspack/core"),
	files: ["bundle.js"],
	bootstrap: true,
	detail: true
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

	beforeAll(async () => {
		rimraf.sync(dist);
		await tester.prepare();
	});

	do {
		const prefix = path.basename(name);
		describe(`${prefix}:build`, () => {
			beforeAll(async () => {
				await tester.compile();
			});
			checkBundleFiles(
				"webpack",
				path.join(dist, "webpack"),
				caseConfig.files!
			);
			checkBundleFiles("rspack", path.join(dist, "rspack"), caseConfig.files!);
		});
		describe(`${prefix}:check`, () => {
			beforeAll(async () => {
				compareMap.clear();
				await tester.check(env);
			});
			for (let file of caseConfig.files!) {
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
	} while (tester.next());

	afterAll(async () => {
		await tester.resume();
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
		detail: config.detail ?? true
	});

	return [processor, fileCompareMap] as [
		DiffProcessor,
		Map<string, TFileCompareResult>
	];
}

function checkBundleFiles(name: string, dist: string, files: string[]) {
	describe(`Checking ${name} dist files`, () => {
		for (let file of files) {
			it(`${name}: ${file} should be generated`, () => {
				expect(fs.existsSync(path.join(dist, file))).toBeTruthy();
			});
		}
	});
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
		it(`all modules should be the same`, () => {
			for (let result of getResults().filter(
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
