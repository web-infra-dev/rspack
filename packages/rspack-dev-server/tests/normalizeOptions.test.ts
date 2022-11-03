import type { RspackOptions } from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
import { createCompiler } from "@rspack/core";
import serializer from "jest-serializer-path";

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
});

function match(config: RspackOptions) {
	const compiler = createCompiler(config);
	const server = new RspackDevServer(compiler);
	expect(server.options).toMatchSnapshot();
}

function matchAdditionEntries(config: RspackOptions) {
	const compiler = createCompiler(config);
	const server = new RspackDevServer(compiler);
	expect(compiler.options.entry).toMatchSnapshot();
}
