import { RspackOptions, rspack } from "@rspack/core";
import { RspackDevServer, Configuration } from "@rspack/dev-server";
import ReactRefreshPlugin from "@rspack/plugin-react-refresh";
import customConfig from "./fixtures/provide-plugin-custom/webpack.config";
// @ts-expect-error
import serializer from "jest-serializer-path";

expect.addSnapshotSerializer(serializer);

// The aims of use a cutstom value rather than
// default is to avoid stack overflow trigged
// by `webpack/schemas/WebpackOption.check.js` in debug mode
const ENTRY = "./placeholder.js";
const ENTRY1 = "./placeholder1.js";

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
		expect(
			await getAdditionEntries({}, { entry: ["something"] })
		).toMatchSnapshot();
	});

	it("shouldn't have reactRefreshEntry.js by default when in production mode", async () => {
		const reactRefreshEntry =
			"<prefix>/rspack-plugin-react-refresh/client/reactRefreshEntry.js";
		const entries1 = await getAdditionEntries(
			{},
			{
				mode: "production",
				entry: ["something"]
			}
		);
		expect(entries1["undefined"]).not.toContain(reactRefreshEntry);
		const entries2 = await getAdditionEntries(
			{},
			{
				mode: "production",
				entry: ["something"],
				plugins: [new ReactRefreshPlugin({ forceEnable: true })]
			}
		);
		expect(entries2["undefined"]).toContain(reactRefreshEntry);
		const entries3 = await getAdditionEntries(
			{},
			{
				mode: "development",
				entry: ["something"],
				plugins: [new ReactRefreshPlugin()]
			}
		);
		expect(entries3["undefined"]).toContain(reactRefreshEntry);
		const entries4 = await getAdditionEntries(
			{},
			{
				mode: "production",
				entry: ["something"],
				plugins: [new ReactRefreshPlugin()]
			}
		);
		expect(entries4["undefined"]).not.toContain(reactRefreshEntry);
	});

	it("should apply HMR plugin by default", async () => {
		const compiler = rspack({
			entry: ENTRY,
			stats: "none"
		});
		const server = new RspackDevServer({}, compiler);
		await server.start();
		const hmrPlugins = compiler.__internal__builtinPlugins.filter(
			p => p.name === "HotModuleReplacementPlugin"
		);
		expect(hmrPlugins.length).toBe(1);
		expect(server.options.hot).toBe(true);
		await server.stop();
	});

	it("should support multi-compiler", async () => {
		const compiler = rspack([
			{
				entry: ENTRY,
				stats: "none"
			},
			{
				entry: ENTRY1,
				stats: "none"
			}
		]);
		const server = new RspackDevServer({}, compiler);
		await server.start();
		await server.stop();
	});

	it("should support custom client transport", async () => {
		const compiler = rspack(customConfig);
		const devServerOptions = {
			client: {
				webSocketTransport: require.resolve(
					"./fixtures/custom-client/CustomSockJSClient"
				)
			},
			webSocketServer: "sockjs"
		};
		const server = new RspackDevServer(devServerOptions, compiler);
		await server.start();
		await server.stop();
	});
});

async function match(config: RspackOptions) {
	const compiler = rspack({
		...config,
		entry: ENTRY,
		stats: "none"
	});
	const server = new RspackDevServer(
		compiler.options.devServer ?? {},
		compiler
	);
	await server.start();
	// it will break ci
	//@ts-ignore
	delete server.options.port;
	expect(server.options).toMatchSnapshot();
	await server.stop();
}

async function getAdditionEntries(
	serverConfig: Configuration,
	config: RspackOptions
) {
	const compiler = rspack({
		...config,
		stats: "none",
		entry: ENTRY
	});

	const server = new RspackDevServer(serverConfig, compiler);
	await server.start();
	const entries = compiler.__internal__builtinPlugins
		.filter(p => p.name === "EntryPlugin")
		.map(p => p.options)
		.reduce<Object>((acc, cur: any) => {
			const name = cur.options.name;
			const request = cur.entry;
			// @ts-expect-error
			if (acc[name]) {
				// @ts-expect-error
				acc[name].import.push(request);
			} else {
				// @ts-expect-error
				acc[name] = { import: [request] };
			}
			return acc;
		}, {});
	// some hack for snapshot
	const value = Object.fromEntries(
		Object.entries(entries).map(([key, item]) => {
			// @ts-expect-error
			const replaced = item.import?.map(entry => {
				const array = entry
					.replace(/\\/g, "/")
					.replace(/port=\d+/g, "")
					.split("/");
				return "<prefix>" + "/" + array.slice(-3).join("/");
			});
			return [key, replaced];
		})
	);
	await server.stop();
	return value;
}
