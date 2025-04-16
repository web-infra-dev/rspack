import withRspack from "next-rspack";
import { JsConfigPathsPlugin } from "./jsconfigpathsplugin.js";
import tsconfig from "./tsconfig.json" with { type: "json" };
import path from "node:path";

/** @type {import('next').NextConfig} */
const nextConfig = {
	/* config options here */
	reactStrictMode: true,
	webpack: (config, { buildId, dev, isServer, defaultLoaders, webpack }) => {
		config.resolve.plugins = config.resolve.plugins || [];
		config.resolve.plugins.push(
			new JsConfigPathsPlugin(tsconfig.compilerOptions.paths, {
				baseUrl: tsconfig.compilerOptions.baseUrl
			})
		);
		return config;
	}
};

export default withRspack(nextConfig);
