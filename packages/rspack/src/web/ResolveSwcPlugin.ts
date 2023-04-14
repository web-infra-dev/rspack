import { Compiler } from "../compiler";
import path from "path";
import { compareVersions } from 'compare-versions';

export class ResolveSwcPlugin {
	apply(compiler: Compiler) {
		const swcHelperVersion = require("@swc/helpers/package.json").version;
		if (compareVersions(swcHelperVersion, '0.5.0') === -1) {
			throw new Error("Please bump @swc/helpers to ^0.5.0");
		}
		const swcPath = path.dirname(require.resolve("@swc/helpers/package.json"));
		const refreshPath = path.dirname(require.resolve("react-refresh"));
		// redirect @swc/helpers to rspack, so user don't have to manual install it
		compiler.options.resolve.alias = {
			"@swc/helpers": swcPath,
			"react-refresh": refreshPath,
			...compiler.options.resolve.alias
		};
	}
}
