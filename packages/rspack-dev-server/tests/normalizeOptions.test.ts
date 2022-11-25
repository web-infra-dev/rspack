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

	it("default client when hot enabled", () => {
		matchAdditionEntries({
			entry: ["something"],
			devServer: {
				hot: true
			}
		});
	});

	it("default client when hot disabled", () => {
		matchAdditionEntries({
			entry: ["something"],
			devServer: {
				hot: false
			}
		});
	})

	it("react.development should be true in default when hot enabled", () => {
		const compiler = createCompiler({
			devServer: {
				hot: true
			}
		});
		new RspackDevServer(compiler);
		expect(compiler.options.builtins.react?.development).toBe(true);
		expect(compiler.options.devServer?.hot).toBe(true);
	});


	it("react.development should be true in default when hot enabled 2", () => {
		const compiler = createCompiler({
			devServer: {
				hot: true
			},
			builtins: {
				react: {
					development: false
				}
			}
		});
		new RspackDevServer(compiler);
		expect(compiler.options.builtins.react?.development).toBe(true);
		expect(compiler.options.devServer?.hot).toBe(true);
	});

});

function match(config: RspackOptions) {
	const compiler = createCompiler(config);
	const server = new RspackDevServer(compiler);
	expect(compiler.options.devServer).toMatchSnapshot();
}

function matchAdditionEntries(config: RspackOptions) {
	const compiler = createCompiler(config);
	const server = new RspackDevServer(compiler);
	const entires = Object.entries(compiler.options.entry);
	// some hack for snapshot
	const value = Object.fromEntries(
		entires.map(([key, item]) => {
			const replaced = item.map(entry => {
				const array = entry.replace(/\\/g, "/").split("/");
				return "<prefix>" + "/" + array.slice(-3).join("/");
			});
			return [key, replaced];
		})
	);
	expect(value).toMatchSnapshot();
}
