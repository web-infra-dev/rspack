import rspackEsmDefaultImport, {
	rspack as rspackEsmNamedImport
} from "@rspack/core";
import assert from "node:assert";

type IsFunction<T> = T extends (...args: any[]) => any ? true : false;

// https://github.com/web-infra-dev/rspack/issues/8095
describe("js-api-type should be correct when importing from @rspack/core", () => {
	it.concurrent("esm default import", async () => {
		// rspack has no default export now
		type Falsy = IsFunction<typeof rspackEsmDefaultImport>;
		const falsy: Falsy = false;
		falsy;
		assert(rspackEsmDefaultImport.BannerPlugin);
	});

	it.concurrent("esm named import", async () => {
		type Truthy = IsFunction<typeof rspackEsmNamedImport>;
		const truthy: Truthy = true;
		truthy;
		assert(rspackEsmNamedImport.BannerPlugin);
		assert(typeof rspackEsmNamedImport === "function");
		const compiler = rspackEsmNamedImport({});
		assert(compiler);
	});

	it.concurrent("rspack.default should not exist in esm import", async () => {
		assert(!(rspackEsmNamedImport as any).default);
		assert(!(rspackEsmDefaultImport as any).default);
	});
});
