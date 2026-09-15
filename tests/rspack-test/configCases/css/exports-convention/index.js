import * as styles1 from "./style.module.css?camel-case#1";
import * as styles2 from "./style.module.css?camel-case#2";

const prod = process.env.NODE_ENV === "production";
const target = process.env.TARGET;

it("concatenation and mangling should work", () => {
	expect(styles1.class).toBe(prod ? "lOQLn3" : "style_module_css_camel-case_1-class");
	expect(styles1["default"]).toBe(prod ? "Nbrg_B" : "style_module_css_camel-case_1-default");
	expect(styles1.fooBar).toBe(prod ? "_8dq8cD" : "style_module_css_camel-case_1-foo_bar");
	expect(styles1.foo_bar).toBe(prod ? "_8dq8cD" : "style_module_css_camel-case_1-foo_bar");

	if (prod) {
		expect(styles2).toMatchObject({
			'btn-info_is-disabled': '_9becks',
			btnInfoIsDisabled: '_9becks',
			'btn--info_is-disabled_1': 'rwFz5i',
			btnInfoIsDisabled1: 'rwFz5i',
			simple: 'lj1EFd',
			foo: 'bar',
			'my-btn-info_is-disabled': 'value',
			myBtnInfoIsDisabled: 'value',
			foo_bar: 'bLCREl',
			fooBar: 'bLCREl',
			class: 'Kdl_B5',
			default: 'bSmOSH'
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
		expect(asIs).toMatchFileSnapshotSync(`${__SNAPSHOT__}/as-is.${__STATS_I__}.txt`);
		expect(camelCase).toMatchFileSnapshotSync(`${__SNAPSHOT__}/camel-case.${__STATS_I__}.txt`);
		expect(camelCaseOnly).toMatchFileSnapshotSync(`${__SNAPSHOT__}/camel-case-only.${__STATS_I__}.txt`);
		expect(dashes).toMatchFileSnapshotSync(`${__SNAPSHOT__}/dashes.${__STATS_I__}.txt`);
		expect(dashesOnly).toMatchFileSnapshotSync(`${__SNAPSHOT__}/dashes-only.${__STATS_I__}.txt`);
		// expect(upper).toMatchSnapshot('upper');
	})
});
