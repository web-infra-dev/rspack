import fs from "fs-extra";
import path from "path";
import { Tester } from "../tester";
import { DiffProcessor, IDiffProcessorOptions } from "../processor";
import rimraf from "rimraf";
import { ECompareResultType, TCompareResult } from "../helper";

export function createDiffCase(name: string, src: string, dist: string) {
	const caseConfigFile = path.join(src, "test.config.js");
	const caseConfig: IDiffProcessorOptions = fs.existsSync(caseConfigFile)
		? require(caseConfigFile)
		: {};

	let moduleCompareResults: TCompareResult[] = [];
	let runtimeCompareResults: TCompareResult[] = [];

	const processor = new DiffProcessor({
		modules: caseConfig.modules,
		onCompareModules(results) {
			moduleCompareResults = results;
		},
		runtimeModules: caseConfig.runtimeModules,
		onCompareRuntimeModules(results) {
			runtimeCompareResults = results;
		},
		ignoreModuleId: caseConfig.ignoreModuleId ?? true,
		ignoreModuleArugments: caseConfig.ignoreModuleArugments ?? true,
		ignorePropertyQuotationMark: caseConfig.ignorePropertyQuotationMark ?? true
	});

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
				it(`webpack bundled script should be generated`, () => {
					expect(
						fs.existsSync(path.join(dist, "webpack", "bundle.js"))
					).toBeTruthy();
				});
				it(`rspack bundled script should be generated`, () => {
					expect(
						fs.existsSync(path.join(dist, "rspack", "bundle.js"))
					).toBeTruthy();
				});
			});
			describe(`${prefix}check`, () => {
				beforeAll(async () => {
					moduleCompareResults = [];
					runtimeCompareResults = [];
					await tester.check();
				});
				describe("compare modules", () => {
					it("should not have rspack only modules", () => {
						expect(
							moduleCompareResults
								.filter(i => i.type === ECompareResultType.OnlyRspack)
								.map(i => i.name)
						).toEqual([]);
					});
					it("should not have webpack only modules", () => {
						expect(
							moduleCompareResults
								.filter(i => i.type === ECompareResultType.OnlyWebpack)
								.map(i => i.name)
						).toEqual([]);
					});
					it("should not have missing modules", () => {
						expect(
							moduleCompareResults
								.filter(i => i.type === ECompareResultType.Missing)
								.map(i => i.name)
						).toEqual([]);
					});
					it(`all modules should be the same`, () => {
						for (let result of moduleCompareResults.filter(
							i => i.type === ECompareResultType.Different
						)) {
							console.log(`${result.name}:\n${result.detail}`);
						}
						expect(
							moduleCompareResults.every(
								i => i.type === ECompareResultType.Same
							)
						).toEqual(true);
					});
				});
				describe("compare runtime modules", () => {
					it("should not have rspack only modules", () => {
						expect(
							runtimeCompareResults
								.filter(i => i.type === ECompareResultType.OnlyRspack)
								.map(i => i.name)
						).toEqual([]);
					});
					it("should not have webpack only modules", () => {
						expect(
							runtimeCompareResults
								.filter(i => i.type === ECompareResultType.OnlyWebpack)
								.map(i => i.name)
						).toEqual([]);
					});
					it("should not have missing modules", () => {
						expect(
							runtimeCompareResults
								.filter(i => i.type === ECompareResultType.Missing)
								.map(i => i.name)
						).toEqual([]);
					});
					it(`all modules should be the same`, () => {
						for (let result of runtimeCompareResults.filter(
							i => i.type === ECompareResultType.Different
						)) {
							console.log(`${result.name}:\n${result.detail}`);
						}
						expect(
							runtimeCompareResults.every(
								i => i.type === ECompareResultType.Same
							)
						).toEqual(true);
					});
				});
			});
		} while (tester.next());

		afterAll(async () => {
			await tester.resume();
		});
		beforeAll(async () => {});
	});
}
