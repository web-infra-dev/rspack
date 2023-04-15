// Setup everything before pnpm install
import { spawnSync } from "child_process";
import * as path from "path";

/**
 *
 * @param {string} context
 * @param {(...args: any[]) => any} fn
 */
function runInContext(context, fn) {
	console.log(`⏺️ Running \`${context}\``);
	const status = fn();
	console.log(`⏹️ Finish  \`${context}\` with ${status}`);
}

try {
	// Make sure developers are at the workspace root
	const { default: rootPkgJson } = await import(
		path.join(process.cwd(), "package.json"),
		{
			assert: {
				type: "json"
			}
		}
	);
	if (rootPkgJson.name != "monorepo") {
		throw new Error(`Unexpected cwd ${process.cwd()}`);
	}
} catch (oldErr) {
	const err = new Error(
		`Make sure you are in workspace root to run this script`
	);
	// @ts-expect-error
	err.cause = oldErr;
	throw err;
}

runInContext(
	"corepack enable",
	() =>
		spawnSync("corepack", ["enable"], {
			cwd: process.cwd(),
			env: process.env,
			stdio: "inherit",
			encoding: "utf-8"
		}).status
);

runInContext(
	"corepack prepare pnpm@8.2.0",
	() =>
		spawnSync("corepack", ["prepare", "pnpm@8.2.0", "--activate"], {
			cwd: process.cwd(),
			env: process.env,
			stdio: "inherit",
			encoding: "utf-8"
		}).status
);