const path = require("path");
const fs = require("fs");
const TOML = require("@iarna/toml");

const swc_version = "v1.3.11";
const swc_packages = [/^swc/];
const crates_dir = path.resolve(__dirname, "../crates");

function readCargoToml(name) {
	const cargoToml = fs
		.readFileSync(path.join(crates_dir, name, "Cargo.toml"))
		.toString();

	return TOML.parse(cargoToml);
}

function writeCargoToml(name, obj) {
	const content = TOML.stringify(obj);
	fs.writeFileSync(path.join(crates_dir, name, "Cargo.toml"), content);
}

const pkg_version_cache = {};
async function getPkgVersion(name) {
	if (!pkg_version_cache[name]) {
		const controller = new AbortController();
		const id = setTimeout(() => controller.abort(), 10000);
		const res = await fetch(
			`https://raw.githubusercontent.com/swc-project/swc/${swc_version}/crates/${name}/Cargo.toml`,
			{
				signal: controller.signal
			}
		);
		clearTimeout(id);
		const tomlString = await res.text();
		const version = TOML.parse(tomlString).package.version;
		pkg_version_cache[name] = version;
	}

	return pkg_version_cache[name];
}

async function updateDependencies(name, deps = {}) {
	const swc_pkgs = Object.keys(deps).filter(pkg =>
		swc_packages.some(item => item.test(pkg))
	);
	let hasChanged = false;
	for (const pkg of swc_pkgs) {
		const version = await getPkgVersion(pkg);
		const oldVersion =
			typeof deps[pkg] === "string" ? deps[pkg] : deps[pkg].version;

		if (version !== oldVersion) {
			console.log(`update ${name} ${pkg} ${oldVersion} -> ${version}`);
			hasChanged = true;
			if (typeof deps[pkg] === "string") {
				deps[pkg] = version;
			} else {
				deps[pkg] = { ...deps[pkg], version };
			}
		}
	}
	return hasChanged;
}

async function main() {
	const crates = fs.readdirSync(crates_dir);
	for (const name of crates) {
		const cargoToml = readCargoToml(name);
		const hasChanged = [
			await updateDependencies(name, cargoToml.dependencies),
			await updateDependencies(name, cargoToml["dev-dependencies"]),
			await updateDependencies(name, cargoToml["build-dependencies"])
		].some(item => !!item);

		if (hasChanged) {
			writeCargoToml(name, cargoToml);
		}
	}
}

main();
