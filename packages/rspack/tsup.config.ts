import { defineConfig } from "tsup";
import prebundleConfig from "./prebundle.config.mjs";

const aliasCompiledPlugin = {
	name: "alias-compiled-plugin",
	setup(build) {
		const { dependencies } = prebundleConfig;

		for (const item of dependencies) {
			const depName = typeof item === "string" ? item : item.name;
			build.onResolve({ filter: new RegExp(`^${depName}$`) }, () => ({
				path: `../compiled/${depName}/index.js`,
				external: true
			}));
		}
	}
};

export default defineConfig({
	entry: ["./src/index.ts"],
	format: ["cjs"],
	target: "node16",
	esbuildPlugins: [aliasCompiledPlugin]
});
