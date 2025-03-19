import rspackCjsRequire, {
	rspack as rspackCjsNamedRequire
} from "@rspack/core";
import assert from "node:assert";

type IsFunction<T> = T extends (...args: any[]) => any ? true : false;

// https://github.com/web-infra-dev/rspack/issues/8095
describe("js-api-type should be correct when requiring from @rspack/core", () => {
	it.concurrent("cjs default require", async () => {
		// const rspack = require('@rspack/core')
		type Truthy = IsFunction<typeof rspackCjsRequire>;
		const truthy: Truthy = true;
		truthy;
		assert(rspackCjsNamedRequire.BannerPlugin);
		assert(typeof rspackCjsNamedRequire === "function");
		const compiler = rspackCjsNamedRequire({});
		assert(compiler);
	});

	it.concurrent("cjs named require", async () => {
		// const { rspack } = require('@rspack/core')
		type Truthy = IsFunction<typeof rspackCjsNamedRequire>;
		const truthy: Truthy = true;
		truthy;
		assert(rspackCjsNamedRequire.BannerPlugin);
		assert(typeof rspackCjsNamedRequire === "function");
		const compiler = rspackCjsNamedRequire({});
		assert(compiler);
	});

	it.concurrent("rspack.default should not exist in cjs require", async () => {
		assert(!(rspackCjsNamedRequire as any).default);
		assert(!(rspackCjsRequire as any).default);
	});
});
