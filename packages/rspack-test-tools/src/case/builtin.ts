import path from "path";

import { BuiltinProcessor, IBuiltinProcessorOptions } from "../processor";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

const FILTERS: Record<
	string,
	IBuiltinProcessorOptions<ECompilerType>["snapshotFileFilter"]
> = {
	"plugin-css": (file: string) => file.endsWith(".css"),
	"plugin-css-modules": (file: string) =>
		file.endsWith(".css") ||
		(file.endsWith(".js") && !file.includes("runtime")),
	"plugin-html": (file: string) => file.endsWith(".html")
};

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	description(name) {
		return `${name} should match snapshot`;
	},
	steps: ({ name, src }) => {
		const cat = path.basename(path.dirname(src));
		const filter = FILTERS[cat];
		return [
			new BuiltinProcessor({
				name,
				snapshot: "output.snap.txt",
				snapshotFileFilter: filter,
				compilerType: ECompilerType.Rspack,
				configFiles: ["rspack.config.js", "webpack.config.js"]
			})
		];
	}
});

export function createBuiltinCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
