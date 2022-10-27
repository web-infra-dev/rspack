import { RspackOptionsNormalized, Compiler } from ".";
import fs from "graceful-fs";
import { PolyfillBuiltinsPlugin } from "./web/polyfillBuiltins";
import { NodeTargetPlugin } from "./node/NodeTargetPlugin";
export class RspackOptionsApply {
	constructor() {}
	process(options: RspackOptionsNormalized, compiler: Compiler) {
		compiler.outputPath = options.output.path;
		compiler.name = options.name;
		compiler.outputFileSystem = fs;
		if (compiler.options.target.includes("node")) {
			new NodeTargetPlugin().apply(compiler);
		}
		if (compiler.options.builtins.polyfillBuiltins) {
			new PolyfillBuiltinsPlugin().apply(compiler);
		}
	}
}
