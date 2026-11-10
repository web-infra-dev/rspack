/**
 * Modified from https://github.com/webpack/webpack/blob/335344160defe0ae3d59a06da8c510d22cc051ec/test/helpers/browserslistConfigPackages.js
 * MIT Licensed
 */
import fs from "node:fs";
import path from "node:path";

const nodeModules = path.resolve(import.meta.dirname, "../../node_modules");
const packages = {
	"browserslist-config-mycompany": {
		development: ["last 1 version"],
		production: ["ie 9"]
	},
	"browserslist-config-mycompany1": {
		development: ["last 1 version"],
		test: ["ie 9"]
	},
	"browserslist-config-mycompany2": {
		development: ["last 1 version"],
		test: ["ie 9"]
	}
};

export function setup() {
	for (const [name, config] of Object.entries(packages)) {
		const directory = path.join(nodeModules, name);
		fs.mkdirSync(directory, { recursive: true });
		fs.writeFileSync(
			path.join(directory, "index.js"),
			`module.exports = ${JSON.stringify(config, null, 2)};\n`
		);
		fs.writeFileSync(
			path.join(directory, "package.json"),
			JSON.stringify({ name, version: "1.0.0", main: "index.js" }, null, 2)
		);
	}

}

export function teardown() {
	for (const name of Object.keys(packages)) {
		fs.rmSync(path.join(nodeModules, name), { recursive: true, force: true });
	}
}
