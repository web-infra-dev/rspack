// Setup everything before pnpm install
import { spawnSync } from "child_process";

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

await import("./check_is_workspace_root");

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
