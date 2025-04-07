// This script copies the tinypool package from node_modules to the compiled directory.
// It removes the engines field from package.json and ensures that the package is working under `engine-strict=true` on lower node versions.

const fs = require("node:fs");
const path = require("node:path");

const tinypool = require
	.resolve("tinypool/package.json")
	.replace("/package.json", "");
const dest = path.resolve(__dirname, "./compiled/tinypool");

fs.cpSync(tinypool, path.resolve(__dirname, "./compiled/tinypool"), {
	recursive: true
});

const pkg = JSON.parse(fs.readFileSync(path.join(dest, "package.json")));
// Removes restirctions on node version (>= 18)
delete pkg.engines;
fs.writeFileSync(path.join(dest, "package.json"), JSON.stringify(pkg, null, 2));
