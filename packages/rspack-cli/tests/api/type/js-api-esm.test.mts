import rspackEsmDefaultImport, {
	rspack as rspackEsmNamedImport
} from "@rspack/core";
import assert from "node:assert";

type IsFunction<T> = T extends (...args: any[]) => any ? true : false;

describe("js-api-type should be correct when importing from @rspack/core", () => {
	it("esm default import", async () => {
		// rspack has no default export now
		type Falsy = IsFunction<typeof rspackEsmDefaultImport>;
		const falsy: Falsy = false;
		falsy;
		assert(rspackEsmDefaultImport.BannerPlugin);
	});

	it("esm named import", async () => {
		type Truthy = IsFunction<typeof rspackEsmNamedImport>;
		const truthy: Truthy = true;
		truthy;
		assert(rspackEsmNamedImport.BannerPlugin);
		assert(typeof rspackEsmNamedImport === "function");
		const compiler = rspackEsmNamedImport({});
		assert(compiler);
	});
});
