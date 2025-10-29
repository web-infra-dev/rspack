const rspackCjsDefaultRequire = require("@rspack/core");
const { rspack: rspackCjsNamedRequire } = require("@rspack/core");

import assert from "node:assert";

type IsFunction<T> = T extends (...args: any[]) => any ? true : false;

// https://github.com/web-infra-dev/rspack/issues/8095
describe.concurrent(
	"js-api-type should be correct when requiring from @rspack/core",
	() => {
		it("cjs default require", async () => {
			type Truthy = IsFunction<typeof rspackCjsDefaultRequire>;
			const truthy: Truthy = true;
			truthy;
			assert(rspackCjsDefaultRequire.BannerPlugin);
			assert(typeof rspackCjsDefaultRequire === "function");
			const compiler = rspackCjsDefaultRequire({});
			assert(compiler);
		});

		it("cjs named require", async () => {
			type Truthy = IsFunction<typeof rspackCjsNamedRequire>;
			const truthy: Truthy = true;
			truthy;
			assert(rspackCjsNamedRequire.BannerPlugin);
			assert(typeof rspackCjsNamedRequire === "function");
			const compiler = rspackCjsNamedRequire({});
			assert(compiler);
		});

		it("rspack.default should not exist in cjs require", async () => {
			assert(!(rspackCjsNamedRequire as any).default);
			assert(!(rspackCjsDefaultRequire as any).default);
		});
	}
);
