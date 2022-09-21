import { Command } from "commander";
import { build } from "@rspack/core";
import { createServer } from "@rspack/dev-server";

const program = new Command();

program
	.option("--env", "env")
	.command("build", {
		isDefault: true
	})
	.description("Rspack build cli")

	.argument("<config-file>", "rspack config  file path")
	.action(async configPath => {
		const config = require(configPath);
		const stats = await build(config);
		console.log(stats);
	});

program
	.command("dev")
	.description("Rspack build cli")
	.argument("<config-file>", "rspack config file path")
	.action(async configPath => {
		const config = require(configPath);
		const server = await createServer(config);
		await server.start();
	});

program.parse();
