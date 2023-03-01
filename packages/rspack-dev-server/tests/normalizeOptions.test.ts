import type { RspackOptions } from "@rspack/core";
import { RspackDevServer, Configuration } from "@rspack/dev-server";
import { createCompiler } from "@rspack/core";
import serializer from "jest-serializer-path";

expect.addSnapshotSerializer(serializer);

describe("normalize options snapshot", () => {
	it("no options", async () => {
		await match({});
	});

	it("port string", async () => {
		await match({
			devServer: {
				port: "9000"
			}
		});
	});

	it("additional entires should added", async () => {
		await matchAdditionEntries(
			{},
			{
				entry: ["something"]
			}
		);
	});

	it("react-refresh client added when react/refresh enabled", async () => {
		await matchAdditionEntries(
			{},
			{
				entry: ["something"],
				builtins: {
					react: {
						refresh: true
					}
				}
			}
		);
	});

	it("react.development and react.refresh should be true by default when hot enabled", async () => {
		const compiler = createCompiler({
			stats: "none"
		});
		const server = new RspackDevServer(
			{
				hot: true
			},
			compiler
		);
		await server.start();
		expect(compiler.options.builtins.react?.refresh).toBe(true);
		expect(compiler.options.builtins.react?.development).toBe(true);
		await server.stop();
	});

	it("hot should be true by default", async () => {
		const compiler = createCompiler({
			stats: "none"
		});
		const server = new RspackDevServer({}, compiler);
		await server.start();
		expect(compiler.options.devServer?.hot).toBe(true);
		expect(server.options.hot).toBe(true);
		await server.stop();
	});
});

async function match(config: RspackOptions) {
	const compiler = createCompiler({
		...config,
		stats: "none",
		infrastructureLogging: {
			level: "info",
			stream: {
				// @ts-expect-error
				write: () => {}
			}
		}
	});
	const server = new RspackDevServer(
		compiler.options.devServer ?? {},
		compiler
	);
	await server.start();
	expect(server.options).toMatchSnapshot();
	await server.stop();
}

async function matchAdditionEntries(
	serverConfig: Configuration,
	config: RspackOptions
) {
	const compiler = createCompiler({
		...config,
		stats: "none",
		infrastructureLogging: {
			stream: {
				// @ts-expect-error
				write: () => {}
			}
		}
	});
	const server = new RspackDevServer(serverConfig, compiler);
	await server.start();
	const entires = Object.entries(compiler.options.entry);
	// some hack for snapshot
	const value = Object.fromEntries(
		entires.map(([key, item]) => {
			const replaced = item.import?.map(entry => {
				const array = entry.replace(/\\/g, "/").split("/");
				return "<prefix>" + "/" + array.slice(-3).join("/");
			});
			return [key, replaced];
		})
	);
	expect(value).toMatchSnapshot();
	await server.stop();
}
