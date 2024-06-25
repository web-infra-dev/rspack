import { getNextName } from "../release/version.mjs";
import fs from "node:fs";
import path from "node:path";

const depFields = [
	"dependencies",
	"devDependencies",
	"peerDependencies",
	"optionalDependencies"
];

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

	for (let field of depFields) {
		if (!pkgJson[field]) {
			continue;
		}
		for (let [depName, _v] of Object.entries(pkgJson[field])) {
			if (depName.startsWith("@rspack/")) {
				if (version.includes("-canary")) {
					delete pkgJson[field][depName];
					pkgJson[field][getNextName(depName)] = version;
				} else {
					pkgJson[field][depName] = version;
				}
			}
		}
	}

	fs.writeFileSync(pkgPath, JSON.stringify(pkgJson, null, 2), "utf8");

	console.log(`Updated rspack related package to ${version} in ${pkgPath}`);
}
