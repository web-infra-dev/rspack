const path = require("path");
const fs = require("fs");
const TOML = require("@iarna/toml");

const swc_packages = [
	{
		regex: /^swc_plugin_import$/,
		getTomlUrl: () =>
			`https://raw.githubusercontent.com/web-infra-dev/swc-plugins/main/crates/plugin_import/Cargo.toml`
	},
	{
		regex: /^swc_emotion$/,
		getTomlUrl: () =>
			`https://raw.githubusercontent.com/swc-project/plugins/main/packages/emotion/transform/Cargo.toml`
	},
	{
		regex: /^swc/,
		getTomlUrl: name =>
			`https://raw.githubusercontent.com/swc-project/swc/v1.3.40/crates/${name}/Cargo.toml`
	}
];

function getPkgTomlUrl(name) {
	for (const { regex, getTomlUrl } of swc_packages) {
		if (regex.test(name)) {
			return getTomlUrl(name);
		}
	}

	return null;
}

function readCargoToml(filePath) {
	const cargoToml = fs.readFileSync(filePath).toString();

	return TOML.parse(cargoToml);
}

function writeCargoToml(filePath, obj) {
	const content = TOML.stringify(obj);
	fs.writeFileSync(filePath, content);
}

const pkg_version_cache = {};
async function getPkgVersion(tomlUrl) {
	if (!pkg_version_cache[tomlUrl]) {
		const res = await fetch(tomlUrl);
		const tomlString = await res.text();
		const version = TOML.parse(tomlString).package.version;
		pkg_version_cache[tomlUrl] = version;
	}

	return pkg_version_cache[tomlUrl];
}

async function updateDependencies(deps = {}) {
	let hasChanged = false;
	for (const pkg of Object.keys(deps)) {
		const tomlUrl = getPkgTomlUrl(pkg);
		if (!tomlUrl) {
			continue;
		}
		const version = await getPkgVersion(tomlUrl);
		const oldVersion =
			typeof deps[pkg] === "string" ? deps[pkg] : deps[pkg].version;

		if (version !== oldVersion) {
			console.log(`update ${pkg} ${oldVersion} -> ${version}`);
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
	const configPath = path.resolve(__dirname, "../Cargo.toml");
	const cargoToml = readCargoToml(configPath);
	const hasChanged = [
		await updateDependencies(cargoToml.workspace.dependencies),
		await updateDependencies(cargoToml.workspace["dev-dependencies"]),
		await updateDependencies(cargoToml.workspace["build-dependencies"])
	].some(item => !!item);

	if (hasChanged) {
		writeCargoToml(configPath, cargoToml);
	}
}

main();
