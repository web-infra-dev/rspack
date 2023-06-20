import path from "path";
import fs from "fs-extra";
import { Fixtures } from "@playwright/test";

type PathInfo = {
	testFile: string;
	testProjectDir: string;
	tempProjectDir: string;
};

export type PathInfoFixtures = {
	pathInfo: PathInfo;
};

type PathInfoWorkerFixtures = {
	_calcPathInfo: (testFile: string) => Promise<PathInfo>;
};

const tempDir = path.resolve(__dirname, "../temp");
async function calcPathInfo(
	testFile: string,
	workerId: string
): Promise<PathInfo> {
	const testProjectDir = path.dirname(testFile);
	const isRspackConfigExist = await fs.exists(
		path.join(testProjectDir, "rspack.config.js")
	);
	if (!isRspackConfigExist) {
		throw new Error(`rspack config not exist in ${testProjectDir}`);
	}

	const tempProjectDir = path.join(tempDir, workerId);
	if (await fs.exists(tempProjectDir)) {
		await fs.remove(tempProjectDir);
	}
	await fs.copy(testProjectDir, tempProjectDir);

	return {
		testFile,
		testProjectDir,
		tempProjectDir
	};
}

export const pathInfoFixtures: Fixtures<
	PathInfoFixtures,
	PathInfoWorkerFixtures
> = {
	pathInfo: async function ({ _calcPathInfo }, use, { file }) {
		const pathInfo = await _calcPathInfo(file);
		await use(pathInfo);
	},

	_calcPathInfo: [
		async function ({}, use, { workerIndex }) {
			let pathInfo: PathInfo = {
				testFile: "",
				testProjectDir: "",
				tempProjectDir: ""
			};
			await use(async function (testFile: string) {
				if (testFile !== pathInfo.testFile) {
					pathInfo = await calcPathInfo(testFile, String(workerIndex));
				}

				return pathInfo;
			});

			if (pathInfo.tempProjectDir) {
				await fs.remove(pathInfo.tempProjectDir);
			}
		},
		{
			scope: "worker"
		}
	]
};
