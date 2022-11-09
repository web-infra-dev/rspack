import type { RspackOptions } from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
import { createCompiler } from "@rspack/core";
import serializer from "jest-serializer-path";
import os from "os";
expect.addSnapshotSerializer(serializer);

describe("normalize options snapshot", () => {
	it("no options", () => {
		match({});
	});

	it("port string", () => {
		match({
			devServer: {
				port: "9000"
			}
		});
	});

	it("additional entires should added", () => {
		matchAdditionEntries({
			entry: ["something"]
		});
	});

	it("react-refresh added when react/refresh enabled", () => {
		matchAdditionEntries({
			entry: ["something"],
			builtins: {
				react: {
					refresh: true
				}
			}
		});
	});
});

function match(config: RspackOptions) {
	const compiler = createCompiler(config);
	const server = new RspackDevServer(compiler);
	expect(server.options).toMatchSnapshot();
}

function matchAdditionEntries(config: RspackOptions) {
	const compiler = createCompiler(config);
	const server = new RspackDevServer(compiler);
	const entires = Object.entries(compiler.options.entry);
	// some hack for snapshot
	const value = Object.fromEntries(
		entires.map(([key, item]) => {
			const replaced = item.map((entry) => {
				const array = entry.replace(/\\/g, "/").split("/");
				return "<prefix>" + "/" + array.slice(-3).join("/");
			});
			return [key, replaced];
		})
	);
	expect(value).toMatchSnapshot();
}
