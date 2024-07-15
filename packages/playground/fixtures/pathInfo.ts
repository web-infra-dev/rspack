import path from "path";
import fs from "fs-extra";
import type { Fixtures } from "@playwright/test";

type PathInfo = {
	testFile: string;
	testProjectDir: string;
	tempProjectDir: string;
};

export type PathInfoFixtures = {
	pathInfo: PathInfo;
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

export const pathInfoFixtures: Fixtures<PathInfoFixtures> = {
	pathInfo: async ({}, use, { file, workerIndex }) => {
		const pathInfo: PathInfo = await calcPathInfo(file, String(workerIndex));
		await use(pathInfo);
		await fs.remove(pathInfo.tempProjectDir);
	}
};
