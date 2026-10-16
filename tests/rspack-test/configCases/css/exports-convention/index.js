import * as styles1 from "./style.module.css?camel-case#1";
import * as styles2 from "./style.module.css?camel-case#2";

const prod = process.env.NODE_ENV === "production";
const target = process.env.TARGET;

it("concatenation and mangling should work", () => {
	expect(styles1.fooBar).toBe(styles1.foo_bar);

	if (prod) {
		const suffix = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK ? "-rspack" : "";
		expect(styles1).toMatchFileSnapshotSync(`${__SNAPSHOT__}/static-camel-case-1.${__STATS_I__}${suffix}.txt`);
		expect(styles2).toMatchFileSnapshotSync(`${__SNAPSHOT__}/static-camel-case-2.${__STATS_I__}${suffix}.txt`);

		expect(Object.keys(__webpack_modules__).length).toBe(target === "web" ? 8 : 1)
	} else {
		expect(styles1.class).toBe("style_module_css_camel-case_1-class");
		expect(styles1["default"]).toBe("style_module_css_camel-case_1-default");
		expect(styles1.fooBar).toBe("style_module_css_camel-case_1-foo_bar");
	}
});

it("should have correct convention for css exports name", async () => {
	await Promise.all([
		import("./style.module.css?as-is"),
		import("./style.module.css?camel-case"),
		import("./style.module.css?camel-case-only"),
		import("./style.module.css?dashes"),
		import("./style.module.css?dashes-only"),
		// import("./style.module.css?upper"),
	]).then(([asIs, camelCase, camelCaseOnly, dashes, dashesOnly, upper]) => {
		expect(asIs).toMatchFileSnapshotSync(`${__SNAPSHOT__}/as-is.${__STATS_I__}.txt`);
		expect(camelCase).toMatchFileSnapshotSync(`${__SNAPSHOT__}/camel-case.${__STATS_I__}.txt`);
		expect(camelCaseOnly).toMatchFileSnapshotSync(`${__SNAPSHOT__}/camel-case-only.${__STATS_I__}.txt`);
		expect(dashes).toMatchFileSnapshotSync(`${__SNAPSHOT__}/dashes.${__STATS_I__}.txt`);
		expect(dashesOnly).toMatchFileSnapshotSync(`${__SNAPSHOT__}/dashes-only.${__STATS_I__}.txt`);
		// expect(upper).toMatchSnapshot('upper');
	})
});
