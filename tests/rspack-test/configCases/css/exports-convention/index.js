import * as styles1 from "./style.module.css?camel-case#1";
import * as styles2 from "./style.module.css?camel-case#2";

const prod = process.env.NODE_ENV === "production";
const target = process.env.TARGET;

const path = __non_webpack_require__("path");

it("concatenation and mangling should work", () => {
	expect(styles1.class).toBe(prod ? "_94e40b9f7fcee9c9" : "_style_module_css_camel-case_1-class");
	expect(styles1["default"]).toBe(prod ? "_35bae0fc11bda5c1" : "_style_module_css_camel-case_1-default");
	expect(styles1.fooBar).toBe(prod ? "f1dabc703f204afd" : "_style_module_css_camel-case_1-foo_bar");
	expect(styles1.foo_bar).toBe(prod ? "f1dabc703f204afd" : "_style_module_css_camel-case_1-foo_bar");

	if (prod) {
		expect(styles2).toMatchObject({
			'btn-info_is-disabled': 'f5b79c92c602eda6',
			btnInfoIsDisabled: 'f5b79c92c602eda6',
			'btn--info_is-disabled_1': 'af0173e628e9cfe2',
			btnInfoIsDisabled1: 'af0173e628e9cfe2',
			simple: '_963d4415d59a2388',
			foo: 'bar',
			'my-btn-info_is-disabled': 'value',
			myBtnInfoIsDisabled: 'value',
			foo_bar: '_6cb0911256f4876f',
			fooBar: '_6cb0911256f4876f',
			class: '_29d97f0793c21a32',
			default: '_6d298e487bdfa215'
		});

		expect(Object.keys(__webpack_modules__).length).toBe(target === "web" ? 8 : 1)
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
		expect(asIs).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `as-is.${__STATS_I__}.txt`));
		expect(camelCase).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `camel-case.${__STATS_I__}.txt`));
		expect(camelCaseOnly).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `camel-case-only.${__STATS_I__}.txt`));
		expect(dashes).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `dashes.${__STATS_I__}.txt`));
		expect(dashesOnly).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `dashes-only.${__STATS_I__}.txt`));
		// expect(upper).toMatchSnapshot('upper');
	})
});
