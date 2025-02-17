import { type Compiler, type Compilation, rspack } from "@rspack/core";
import { beforeAll, bench, describe } from "vitest";
import rspackConfig from "./fixtures/ts-react/rspack.config";

let thisCompiler: Compiler;
let theCompilation: Compilation;

beforeAll(() => {
	return new Promise((resolve, reject) => {
		thisCompiler = rspack(
			{
				...rspackConfig,
				mode: "production",
				plugins: [
					...(rspackConfig.plugins ?? []),
					compiler => {
						compiler.hooks.compilation.tap("PLUGIN", compilation => {
							theCompilation = compilation;
						});
					}
				]
			}
		);
        thisCompiler.run((err, stats) => {
            if (err) {
                reject(err);
            }
            if (stats?.hasErrors()) {
                reject(new Error(stats.toString({})));
            }
            resolve(undefined);
        });
        for (const module of theCompilation.modules) {}
    });
});

beforeEach(() => {
    return new Promise((resolve, reject) => {
        thisCompiler.run((err, stats) => {
            if (err) {
                reject(err);
            }
            if (stats?.hasErrors()) {
                reject(new Error(stats.toString({})));
            }
            resolve(undefined);
        });
    });
});

describe("TypeScript React project HMR", () => {
    // Rspack reuses the Module objects created on the JavaScript side during the previous compilation phase in HMR.
    // Therefore, module traversal during HMR is faster compared to the build process.
    bench("traverse compilation.modules (HMR)", () => {
        for (const module of theCompilation.modules) {
            module.identifier();
        }
    });
});
