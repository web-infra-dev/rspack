import path from "path";
import fse from "fs-extra";
import crypto from "crypto";
import cp from "child_process";
async function copyFixture(
	targetDir: string,
	fixtureName: string
): Promise<void> {
	const base = path.resolve(__dirname, "../e2e-fixtures");
	const fixturePath = path.resolve(base, fixtureName);
	return fse.pathExists(fixturePath).then(p => {
		if (!p) {
			throw new Error(
				`Could not find fixture with name "${fixtureName}" in ${base}`
			);
		}
		return fse.copy(fixturePath, targetDir);
	});
}

async function createTempDir(targetDir: string, tag?: string): Promise<string> {
	const id = crypto.randomUUID();
	/**
	 * ```
	 * xxx/xxxx/<fixtureName>_<tag>_<id>
	 * ```
	 */
	const cwd = tag ? `${targetDir}_${tag}_${id}` : `${targetDir}_${id}`;
	return fse.ensureDir(cwd).then(() => cwd);
}

export async function initFixture(
	fixtureName: string,
	tag?: string
): Promise<string> {
	const targetDir = path.resolve(
		__dirname,
		"../../",
		".test-temp",
		fixtureName
	);
	const tempDir = await createTempDir(targetDir, tag);
	return Promise.resolve()
		.then(() => copyFixture(tempDir, fixtureName))
		.then(() => tempDir);
}

export async function installDeps(cwd: string) {
	return new Promise(resolve => {
		cp.exec("npm install", { cwd }, (error, _stdout, stderr) => {
			if (error || stderr) {
				throw Error(`Install failed in ${cwd}`);
			} else {
				resolve(undefined);
			}
		});
	});
}
