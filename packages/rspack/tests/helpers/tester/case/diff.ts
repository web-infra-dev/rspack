import fs from "fs-extra";
import path from "path";
import { Tester } from "../tester";
import {
	DiffProcessor,
	IDiffProcessorOptions,
	OUTPUT_MAIN_FILE
} from "../processor";
import rimraf from "rimraf";
import { ECompareResultType, TModuleCompareResult } from "../helper";

const DEFAULT_CASE_CONFIG: IDiffProcessorOptions = {
	files: [OUTPUT_MAIN_FILE],
	ignorePropertyQuotationMark: true,
	ignoreModuleId: true,
	ignoreModuleArugments: false
};

type TFileCompareResult = {
	modules: TModuleCompareResult[];
	runtimeModules: TModuleCompareResult[];
};

export function createDiffCase(name: string, src: string, dist: string) {
	const caseConfigFile = path.join(src, "test.config.js");
	const caseConfig: IDiffProcessorOptions = Object.assign(
		{},
		DEFAULT_CASE_CONFIG,
		fs.existsSync(caseConfigFile) ? require(caseConfigFile) : {}
	);

	const [processor, compareMap] = createDiffProcessor(caseConfig);
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [processor]
	});

	describe(name, () => {
		beforeAll(async () => {
			rimraf.sync(dist);
			await tester.prepare();
		});

		do {
			const prefix = `[${name}][${tester.step + 1}]:`;
			describe(`${prefix}build`, () => {
				beforeAll(async () => {
					await tester.compile();
				});
				checkBundleFiles(
					"webpack",
					path.join(dist, "webpack"),
					caseConfig.files!
				);
				checkBundleFiles(
					"rspack",
					path.join(dist, "rspack"),
					caseConfig.files!
				);
			});
			describe(`${prefix}check`, () => {
				beforeAll(async () => {
					compareMap.clear();
					await tester.check();
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
			});
		} while (tester.next());

		afterAll(async () => {
			await tester.resume();
		});
	});
}

export function createDiffProcessor(config: IDiffProcessorOptions) {
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
		files: config.files,
		modules: config.modules,
		runtimeModules: config.runtimeModules,
		ignoreModuleId: config.ignoreModuleId ?? true,
		ignoreModuleArugments: config.ignoreModuleArugments ?? true,
		ignorePropertyQuotationMark: config.ignorePropertyQuotationMark ?? true,
		onCompareModules: createCompareResultHandler("modules"),
		onCompareRuntimeModules: createCompareResultHandler("runtimeModules")
	});

	return [processor, fileCompareMap] as [
		DiffProcessor,
		Map<string, TFileCompareResult>
	];
}

export function checkBundleFiles(name: string, dist: string, files: string[]) {
	describe(`Checking ${name} dist files`, () => {
		for (let file of files) {
			it(`${name}: ${file} should be generated`, () => {
				expect(fs.existsSync(path.join(dist, file))).toBeTruthy();
			});
		}
	});
}

export function checkCompareResults(
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
		it("should not have any respack-only module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.OnlyRspack)
					.map(i => i.name)
			).toEqual([]);
		});
		it("should not have any webpack-only module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.OnlyWebpack)
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
