import path from "node:path";
import { fileURLToPath } from "node:url";
import fs from "fs-extra";

import "zx/globals";

$.verbose = true;

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const BENCH_DIR = path.resolve(__dirname, "../../.bench");

fs.ensureDir(BENCH_DIR);

async function rspackBenchcases() {
	if (await fs.exists(path.join(BENCH_DIR, "rspack-benchcases"))) {
		console.log("rspack-benchcases already exists, skipping");
		return;
	}
	await $`git clone --depth=1 https://github.com/rspack-contrib/rspack-benchcases.git ${path.join(BENCH_DIR, "rspack-benchcases")}`;
	Promise.all(
		[".git", ".github"].map(item =>
			fs.remove(path.join(BENCH_DIR, "rspack-benchcases", item))
		)
	);
}

await rspackBenchcases();

await $`pnpm install`;
