import fs from "fs-extra";
import path from "path";
import { Tester } from "../tester";
import { DiffBuilder } from "../processor/diff";
import rimraf from "rimraf";

const buildTimeout = 60 * 1000;
const checkTimeout = 60 * 1000;

export function createDiffCase(name: string, src: string, dist: string) {
	const caseConfigFile = path.join(src, "test.config.js");
	const caseConfig = fs.existsSync(caseConfigFile)
		? require(caseConfigFile)
		: {};
	describe(name, () => {
		const tester = new Tester({
			name,
			src,
			dist,
			steps: [
				new DiffBuilder({
					webpackDist: path.join(dist, "webpack"),
					rspackDist: path.join(dist, "rspack"),
					modules: caseConfig.modules,
					runtimeModules: caseConfig.runtimeModules,
					ignoreModuleId: caseConfig.ignoreModuleId
				})
			]
		});
		beforeAll(async () => {
			rimraf.sync(dist);
			await tester.prepare();
		});

		do {
			it(
				`${name} step ${tester.step + 1} should be compiled`,
				async () => {
					await tester.compile();
				},
				caseConfig.buildTimeout || buildTimeout
			);
			it(
				`${name} step ${tester.step + 1} should be checked`,
				async () => {
					await tester.check();
				},
				caseConfig.checkTimeout || checkTimeout
			);
		} while (tester.next());

		afterAll(async () => {
			await tester.resume();
		});
	});
}
