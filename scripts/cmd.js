const cp = require("child_process");
function createCLi() {
	const { Command } = require("commander");

	const cli = new Command();
	cli
		.name("install")
		.description("install node dependencies")
		.action(() => {
			console.info("start install deps");
			cp.execSync("pnpm i", {
				stdio: "inherit"
			});
			console.info("finish install deps");
		});

	return cli;
}
function main() {
	const cli = createCLi();
	cli.parse(process.argv.slice(2));
}
main();
