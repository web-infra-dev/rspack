import Server from "webpack-dev-server";
import { Server as ModernServer } from "@modern-js/server";
import { rdm } from "@rspack/dev-middleware";
import { RspackCLI } from "@rspack/cli";
import path from "path";
import { rspack } from "@rspack/core";
async function run_express_server() {
	const cli = new RspackCLI();
	const config = await cli.loadConfig({
		config: path.resolve(__dirname, "../example/basic/rspack.config.js")
	});
	const compiler = rspack(config);
	const server = new Server({}, compiler as any);
	const app = await server.start();
}
async function run_modern_server() {
	const cli = new RspackCLI();
	const config = await cli.loadConfig({
		config: path.resolve(__dirname, "../example/basic/rspack.config.js")
	});
	const compiler = rspack(config);
	const server = new ModernServer({
		pwd: config.context,
		dev: {},
		port: 8888,
		compiler: compiler as any,
		config: {
			source: {},
			tools: {
				devServer: {
					before: [rdm(compiler, {})]
				}
			},
			server: {}
		} as any
	});
	const app = await server.init();
	return new Promise<void>(resolve => {
		app.listen(8888, (err: Error) => {
			if (err) {
				throw err;
			}
			resolve();
		});
	});
}
run_modern_server();
