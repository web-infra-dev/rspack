import { RspackOptionsNormalized, Compiler } from ".";
import fs from "graceful-fs";

import { NodeTargetPlugin } from "./node/NodeTargetPlugin";
import { ResolveSwcPlugin } from "./web/ResolveSwcPlugin";
export class RspackOptionsApply {
	constructor() {}
	process(options: RspackOptionsNormalized, compiler: Compiler) {
		compiler.outputPath = options.output.path;
		compiler.name = options.name;
		compiler.outputFileSystem = fs;
		if (compiler.options.target.includes("node")) {
			new NodeTargetPlugin().apply(compiler);
		}
		new ResolveSwcPlugin().apply(compiler);
	}
}
