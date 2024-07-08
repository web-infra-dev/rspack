import * as path from "path";

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
	// @ts-expect-error error.cause is introduced in ES2022, ignore here
	err.cause = oldErr;
	throw err;
}

export {};
