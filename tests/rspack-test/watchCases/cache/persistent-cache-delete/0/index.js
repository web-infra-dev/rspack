import { existsSync, readdirSync, rmSync } from "fs";
import value from "./value";

const cacheDir = __CACHE_DIR__;

function sleep(ms) {
	return new Promise(resolve => setTimeout(resolve, ms));
}

function listCacheFiles() {
	if (!existsSync(cacheDir)) {
		return [];
	}

	const files = [];
	const walk = dir => {
		let entries;
		try {
			entries = readdirSync(dir, { withFileTypes: true });
		} catch (error) {
			if (error && error.code === "ENOENT") {
				return;
			}
			throw error;
		}
		for (const entry of entries) {
			const file = `${dir}/${entry.name}`;
			if (entry.isDirectory()) {
				walk(file);
			} else {
				files.push(file);
			}
		}
	};
	walk(cacheDir);
	return files;
}

function hasCommittedPackFile() {
	return listCacheFiles().some(
		file => file.endsWith(".pack") && !file.includes("/.temp/")
	);
}

async function waitForPackFiles() {
	const start = Date.now();
	while (Date.now() - start < 5000) {
		if (hasCommittedPackFile()) {
			return;
		}
		await sleep(100);
	}
	throw new Error("Timed out waiting for persistent cache pack files");
}

async function removeCacheDir() {
	const start = Date.now();
	while (Date.now() - start < 5000) {
		try {
			rmSync(cacheDir, { recursive: true, force: true });
			if (listCacheFiles().length === 0) {
				return;
			}
		} catch (error) {
			if (error && error.code !== "ENOTEMPTY") {
				throw error;
			}
		}
		await sleep(100);
	}
	throw new Error("Timed out removing persistent cache directory");
}

it("should keep writing persistent cache after the cache directory is removed", async () => {
	expect(value).toBe(WATCH_STEP);
	await waitForPackFiles();

	if (WATCH_STEP === "0") {
		await removeCacheDir();
		expect(listCacheFiles().length).toBe(0);
	} else {
		expect(WATCH_STEP).toBe("1");
	}
});
