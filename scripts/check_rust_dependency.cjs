// @ts-nocheck

// The code checks for multiple version dependencies in Rust crates.
// It reads the Cargo.toml files of all crates in a specified directory and extracts the dependencies.
// It then uses the cargo tree command to get a list of all dependencies and their versions and
// checks if any of the dependencies have multiple versions.
// If a crate has multiple version dependencies, an error message is pushed to an array. Finally,
// if there are any error messages, they are logged to the console along with a command to run for more information.

// This is a best effort checking. It's possible that some dependencies may be missed, especially if
// they are not listed in the Cargo.toml file or if they are not direct dependencies of the crate.

const path = require("path");
const fs = require("fs");
const child_process = require("child_process");
const TOML = require("@iarna/toml");

const crates_dir = path.resolve(__dirname, "../crates");


// 'bitflags': napi has upgraded to latest `bitfalgs@2.x.x`, but there are still lots of dependencies still use `bitflags@1.x.x`, like `clap, swc`,
// this cause CI failed in version checking, `bitflags@2.x.x` still need some time to adopt in rust community, but we need upgrade napi-rs to latest to fix some bug.
// so bypass `bitflags` for now.
const ignore_deps = ['bitflags'];

function getRepeatDeps() {
	const treeResult = child_process
		.execSync("cargo tree -d --depth=0 -e no-dev")
		.toString();

	// Record<string, string[]>
	const result = {};
	for (const dep of treeResult.split("\n\n")) {
		const [name, version] = dep.split(" ");
		if (ignore_deps.includes(name)) {
			continue;
		}
		if (!result[name]) {
			result[name] = [];
		}
		result[name].push(version);
	}
	return result;
}

async function main() {
	const repeat_deps = getRepeatDeps();

	const error_messages = [];
	const crates = fs.readdirSync(crates_dir);
	for (const name of crates) {
		const cargoStr = fs
			.readFileSync(path.join(crates_dir, name, "Cargo.toml"))
			.toString();
		const toml = TOML.parse(cargoStr);
		const deps = [
			...Object.keys(toml.dependencies || {}),
			...Object.keys(toml["dev-dependencies"] || {}),
			...Object.keys(toml["build-dependencies"] || {})
		];
		for (const dep of deps) {
			if (repeat_deps[dep]) {
				error_messages.push(
					`crate ${name} has multiple version dependence ${dep}(${repeat_deps[
						dep
					].join(", ")})`
				);
			}
		}
	}

	if (error_messages.length) {
		console.error(error_messages.join("\n"));
		console.error("");
		console.error("run 'cargo tree -d' to show more info");
		process.exit(1);
	}
}

main();
