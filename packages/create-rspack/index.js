#!/usr/bin/env node
const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const fs = require("node:fs");
const path = require("node:path");
const prompts = require("prompts");
const { formatTargetDir } = require("./utils");
const { version } = require("./package.json");

yargs(hideBin(process.argv))
	.command("$0", "init rspack project", async argv => {
		const { help } = argv.argv;
		if (help) return;

		const onCancel = () => {
			console.log("Operation cancelled.");
			process.exit(0);
		};

		const defaultProjectName = "rspack-project";
		let template = "react";
		let targetDir = defaultProjectName;

		const promptProjectDir = async () =>
			await prompts(
				[
					{
						type: "text",
						name: "projectDir",
						initial: defaultProjectName,
						message: "Project folder",
						onState: state => {
							targetDir = formatTargetDir(state.value) || defaultProjectName;
						}
					}
				],
				{ onCancel }
			);

		await promptProjectDir();
		let projectDir = getProjectDir(targetDir);
		let root = path.resolve(process.cwd(), projectDir);
		while (fs.existsSync(root)) {
			console.log(
				`${targetDir} is not empty, please choose another project name`
			);
			await promptProjectDir();
			projectDir = getProjectDir(targetDir);
			root = path.resolve(process.cwd(), projectDir);
		}

		// choose template
		await prompts(
			[
				{
					type: "select",
					name: "template",
					message: "Project template",
					choices: [
						{ title: "react", value: "react" },
						{ title: "react-ts", value: "react-ts" },
						{ title: "vue", value: "vue" },
						{ title: "vue-ts", value: "vue-ts" }
					],
					onState: state => {
						template = state.value;
					}
				}
			],
			{ onCancel }
		);

		fs.mkdirSync(root, { recursive: true });
		const srcFolder = path.resolve(__dirname, `template-${template}`);
		copyFolder(srcFolder, root, targetDir);
		const pkgManager = getPkgManager();
		console.log("\nDone. Now run:\n");
		console.log(`cd ${projectDir}\n`);
		console.log(`${pkgManager} install\n`);
		console.log(`${pkgManager} run dev\n`);
	})
	.help()
	.parse();

function copyFolder(src, dst, targetDir) {
	const renameFiles = {
		_gitignore: ".gitignore"
	};

	fs.mkdirSync(dst, { recursive: true });
	for (const file of fs.readdirSync(src)) {
		if (file === "node_modules") {
			continue;
		}
		const srcFile = path.resolve(src, file);
		const dstFile = renameFiles[file]
			? path.resolve(dst, renameFiles[file])
			: path.resolve(dst, file);
		const stat = fs.statSync(srcFile);
		if (stat.isDirectory()) {
			copyFolder(srcFile, dstFile, targetDir);
		} else {
			// use create-rspack version as @rspack/xxx version in template
			if (file === "package.json") {
				const pkg = require(srcFile);
				if (pkg.dependencies) {
					for (const key of Object.keys(pkg.dependencies)) {
						if (key.startsWith("@rspack/")) {
							pkg.dependencies[key] = version;
						}
					}
				}
				if (pkg.devDependencies) {
					for (const key of Object.keys(pkg.devDependencies)) {
						if (key.startsWith("@rspack/")) {
							pkg.devDependencies[key] = version;
						}
					}
				}
				if (pkg.name) {
					pkg.name = getPkgName(targetDir);
				}
				fs.writeFileSync(dstFile, JSON.stringify(pkg, null, 2), "utf-8");
			} else {
				fs.copyFileSync(srcFile, dstFile);
			}
		}
	}
}

function getPkgName(targetDir) {
	const scopeMatch = matchScopedPackageName(targetDir);
	if (scopeMatch) {
		return targetDir; // Scoped package name
	}
	return path.basename(targetDir); // Use the base name of the target directory
}

function getProjectDir(targetDir) {
	const scopeMatch = matchScopedPackageName(targetDir);
	if (scopeMatch) {
		return scopeMatch[1]; // Subdirectory project name for scoped packages
	}
	return targetDir;
}

function matchScopedPackageName(targetDir) {
	return targetDir.match(/^@[^/]+\/(.+)/);
}

function getPkgManager() {
	const ua = process.env.npm_config_user_agent;
	if (!ua) return "npm";
	const [pkgInfo] = ua.split(" ");
	const [name] = pkgInfo.split("/");
	return name || "npm";
}
