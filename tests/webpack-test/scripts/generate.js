const fs = require("fs");
const path = require("path");
const { run } = require("./utils");

const GITHUB_ACTOR = process.env.GITHUB_ACTOR;
const [, , token, commit_sha] = process.argv;
const repoUrl = token
	? `https://${GITHUB_ACTOR}:${token}@github.com/web-infra-dev/rspack.git`
	: "https://github.com/web-infra-dev/rspack";

(async () => {
	const rootDir = path.resolve(__dirname, "../../../");
	const currentDataPath = path.resolve(rootDir, "out.json");
	const currentData = fs.readFileSync(currentDataPath).toString();

	await run("git", ["config", "--global", "user.name", "github-actions[bot]"]);
	await run("git", [
		"config",
		"--global",
		"user.email",
		"41898282+github-actions[bot]@users.noreply.github.com",
	]);

	const targetDir = path.resolve(rootDir, ".gh-pages");
	if (!fs.existsSync(targetDir)) {
		await run("git", [
			"clone",
			"--branch",
			"gh-pages",
			"--single-branch",
			"--depth",
			"1",
			repoUrl,
			".gh-pages",
		]);
	}
	const cwd = process.cwd();
	let dataPath = path.resolve(targetDir, "result.json");
	let indexPath = path.resolve(targetDir, "index.txt");

	let historyJson = "{}";
	let indexContent = "";
	if (fs.existsSync(dataPath)) {
		historyJson = fs.readFileSync(path.resolve(dataPath)).toString();
	}
	if (fs.existsSync(indexPath)) {
		indexContent = fs.readFileSync(path.resolve(indexPath)).toString();
	}

	process.chdir(targetDir);
	await run("git", ["remote", "set-url", "origin", repoUrl]);
	await run("git", ["reset", "--hard", "origin/gh-pages"]);
	await run("git", ["pull", "--rebase"]);

	console.log("== update metric data ==");
	const historyData = JSON.parse(historyJson);
	historyData[commit_sha] = JSON.parse(currentData);
	console.log("== update index data ==");
	indexContent = indexContent.trim() + "\n" + commit_sha.toString();

	fs.writeFileSync(dataPath, JSON.stringify(historyData));
	fs.writeFileSync(indexPath, indexContent);

	console.log("== commit ==");
	await run("git", ["add", "result.json", "index.txt"]);
	await run("git", ["commit", "-m", `"update metric data"`]);

	console.log("== push ==");
	await run("git", ["push"]);
})();

// (async () => {
// 	const targetDir = resolve(rootDir, ".gh-pages");
// 	if (!(await dirExist(targetDir))) {
// 		await run("git", [
// 			"clone",
// 			"--branch",
// 			"gh-pages",
// 			"--single-branch",
// 			"--depth",
// 			"1",
// 			token
// 				? `https://${GITHUB_ACTOR}:${token}@github.com/web-infra-dev/rspack-ecosystem-benchmark.git`
// 				: "https://github.com/web-infra-dev/rspack-ecosystem-benchmark.git",
// 			".gh-pages",
// 		]);
// 	}
// 	process.chdir(targetDir);
// })().catch((err) => {
// 	process.exitCode = 1;
// 	console.error(err.stack);
// });
