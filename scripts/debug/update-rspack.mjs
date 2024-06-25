import { getNextName } from "../release/version.mjs";
import fs from "node:fs";
import path from "node:path";

const depFields = [
	"dependencies",
	"devDependencies",
	"peerDependencies",
	"optionalDependencies"
];

export function getNextPkgJson(pkgJson, version, isSnapshotVersion) {
	const newPkgJson = { ...pkgJson };
	for (let field of depFields) {
		if (!newPkgJson[field]) {
			continue;
		}
		for (let [depName, _v] of Object.entries(newPkgJson[field])) {
			if (depName.startsWith("@rspack/")) {
				if (isSnapshotVersion) {
					delete newPkgJson[field][depName];
					newPkgJson[field][getNextName(depName)] = version;
				} else {
					newPkgJson[field][depName] = version;
				}
			}
		}
	}
	return newPkgJson;
}

export async function update_rspack_handler(version, options) {
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

	const newPkgJson = await getNextPkgJson(
		pkgJson,
		version,
		version.includes("-canary")
	);
	fs.writeFileSync(pkgPath, JSON.stringify(newPkgJson, null, 2), "utf8");

	console.log(`Updated rspack related package to ${version} in ${pkgPath}`);
}
