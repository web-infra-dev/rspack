// Setup everything before pnpm install
import "zx/globals";
import * as path from "path";

// Make sure developers are at the workspace root

try {
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

await $`corepack enable`;
