import type { RspackOptions } from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
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
		await matchAdditionEntries({
			entry: ["something"]
		});
	});

	it("react-refresh client added when react/refresh enabled", async () => {
		await matchAdditionEntries({
			entry: ["something"],
			builtins: {
				react: {
					refresh: true
				}
			}
		});
	});

	it("react.development and react.refresh should be true in default when hot enabled", async () => {
		const compiler = createCompiler({
			stats: "none",
			devServer: {
				hot: true
			}
		});
		const server = new RspackDevServer(
			compiler.options.devServer ?? {},
			compiler
		);
		await server.start();
		expect({
			builtins: compiler.options.builtins,
			devServer: compiler.options.devServer
		}).toMatchSnapshot();
		await server.stop();
		// should pointed to the same memory.
		expect(compiler.options.devServer === server.options).toBeTruthy();
	});
	it("compier.options.devServer should be equal to server.options when devServer is undefined", async () => {
		const compiler = createCompiler({
			stats: "none"
		});
		const server = new RspackDevServer(
			compiler.options.devServer ?? {},
			compiler
		);
		await server.start();
		expect({
			builtins: compiler.options.builtins,
			devServer: compiler.options.devServer
		}).toMatchSnapshot();
		await server.stop();
		// should pointed to the same memory.
		expect(compiler.options.devServer === server.options).toBeTruthy();
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

async function matchAdditionEntries(config: RspackOptions) {
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
	const server = new RspackDevServer(
		compiler.options.devServer ?? {},
		compiler
	);
	await server.start();
	const entires = Object.entries(compiler.options.entry);
	// some hack for snapshot
	const value = Object.fromEntries(
		entires.map(([key, item]) => {
			const replaced = item.import.map(entry => {
				const array = entry.replace(/\\/g, "/").split("/");
				return "<prefix>" + "/" + array.slice(-3).join("/");
			});
			return [key, replaced];
		})
	);
	expect(value).toMatchSnapshot();
	await server.stop();
}
