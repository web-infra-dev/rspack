import { findWorkspacePackagesNoCheck } from "@pnpm/find-workspace-packages";
import fs from "node:fs";
import { getNextName } from "../release/version.mjs";

export async function overrides_rspack_handler(version, options) {
	const root = process.cwd();
	const { path: pathOpts } = options;

	let pkgPath = pathOpts;
	if (typeof pkgPath !== "string") {
		pkgPath = path.resolve(process.cwd(), "package.json");
	}

	if (!path.isAbsolute(pkgPath)) {
		pkgPath = path.resolve(root, pkgPath);
	}

	if (path.basename(pkgPath) !== "package.json") {
		pkgPath = path.resolve(pkgPath, "package.json");
	}

	const pkgJson = (
		await import(pkgPath, {
			assert: {
				type: "json"
			}
		})
	)["default"];

	if (!pkgJson["pnpm"]) {
		pkgJson["pnpm"] = {};
	}

	if (!pkgJson["pnpm"]["overrides"]) {
		pkgJson["pnpm"]["overrides"] = {};
	}

	const workspaces = await findWorkspacePackagesNoCheck(root);
	const workspaceNames = workspaces.map(workspace => workspace.manifest.name);
	const isSnapshot = version.includes("-canary");

	for (const name of workspaceNames) {
		if (name.startsWith("@rspack/")) {
			pkgJson["pnpm"]["overrides"][name] = isSnapshot
				? `npm:${getNextName(name)}@${version}`
				: version;
		}
	}

	fs.writeFileSync(pkgPath, JSON.stringify(pkgJson, null, 2), "utf8");

	console.log(`Added pnpm.overrides to ${pkgPath}`);
}
