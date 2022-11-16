import { Compiler } from "../compiler";
import path from "path";
export class ResolveSwcPlugin {
	apply(compiler: Compiler) {
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
