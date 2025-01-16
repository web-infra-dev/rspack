import { rspack } from "@rspack/core";
import { beforeAll, bench, describe } from "vitest";
import rspackConfig from "./fixtures/vanilla-js/rspack.config";

let theResolveData: any;

beforeAll(() => {
	return new Promise((resolve, reject) =>
		rspack(
			{
				...rspackConfig,
				mode: "production",
				plugins: [
					...(rspackConfig.plugins ?? []),
					compiler => {
						compiler.hooks.contextModuleFactory.tap("PLUGIN", cmf => {
							cmf.hooks.beforeResolve.tapAsync("PLUGIN", (resolveData, callback) => {
                                theResolveData = resolveData
                                resolve(undefined);
                            });
						});
					}
				]
			},
			(err, stats) => {
				if (err) {
					reject(err);
				}
				if (stats?.hasErrors()) {
					reject(new Error(stats.toString({})));
				}
				resolve(undefined);
			}
		)
	);
});

describe("RegExp N-API Benchmarks", () => {
	bench("Send JavaScript RegExp to Rust", () => {
		theResolveData.regExp = /^\.\/solo\-.*\.js$/;
	});

    bench("Send Rust RegExp to JavaScript", () => {
		theResolveData.regExp;
	});
});
