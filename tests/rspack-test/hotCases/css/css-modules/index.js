import * as styles from "./style.module.css";

it("should work", async () => {
	expect(styles).toMatchObject({ class: "_style_module_css-class" });

	let styles2 = await import("./style2.module.css");
	expect(styles2).toMatchObject({
		foo: "_style2_module_css-foo"
	});

	await NEXT_HMR();

	expect(styles).toMatchObject({
		"class-other": "_style_module_css-class-other"
	});
	styles2 = await import("./style2.module.css");
	expect(styles2).toMatchObject({
		"bar": "_style2_module_css-bar"
	});
});

module.hot.accept(["./style.module.css", "./style2.module.css"]);
