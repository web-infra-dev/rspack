import { getCompiler, source, compile } from "./helpers";

it("should enforce esm for empty module without options.esModule", async () => {
	expect.assertions(3);

	const compiler = getCompiler(
		"./esm.js",
		{},
		{
			mode: "production",
			optimization: { minimize: false }
		}
	);
	const stats = await compile(compiler);
	expect(stats.hasErrors()).toBe(false);
	const { modules } = stats.toJson({ all: false, modules: true });
	expect(
		modules.filter(m => m.moduleType !== "runtime" && !m.orphan).length
	).toBe(2);
	expect(source("./simple.css", stats)).toMatchInlineSnapshot(`
		"// extracted by rspack-mini-css-extract-plugin
		export { };
		"
	`);
});

it("should enforce esm for empty module with options.esModule", async () => {
	expect.assertions(3);

	const compiler = getCompiler(
		"./esm.js",
		{ esModule: true },
		{
			mode: "production",
			optimization: { minimize: false }
		}
	);
	const stats = await compile(compiler);
	expect(stats.hasErrors()).toBe(false);
	const { modules } = stats.toJson({ all: false, modules: true });
	expect(
		modules.filter(m => m.moduleType !== "runtime" && !m.orphan).length
	).toBe(2);
	expect(source("./simple.css", stats)).toMatchInlineSnapshot(`
		"// extracted by rspack-mini-css-extract-plugin
		export { };
		"
	`);
});

it('should keep empty module when options.esModule is equal "false"', async () => {
	expect.assertions(3);

	const compiler = getCompiler(
		"./esm.js",
		{ esModule: false },
		{
			mode: "production",
			optimization: { minimize: false }
		}
	);
	const stats = await compile(compiler);
	expect(stats.hasErrors()).toBe(false);
	const { modules } = stats.toJson({ all: false, modules: true });
	expect(
		modules.filter(m => m.moduleType !== "runtime" && !m.orphan).length
	).toBe(2);
	expect(source("./simple.css", stats)).toMatchInlineSnapshot(`
		"// extracted by rspack-mini-css-extract-plugin
		"
	`);
});
