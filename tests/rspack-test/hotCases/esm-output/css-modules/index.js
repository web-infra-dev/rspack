import * as styles from "./style.module.css";

import.meta.webpackHot.accept(["./style.module.css", "./style2.module.css"])

it("should work", async () => {
	expect(styles).toMatchObject({ class: "_style_module_css-class" });
	const styles2 = await import("./style2.module.css");

	expect(styles2).toMatchObject({
		foo: "_style2_module_css-foo"
	});

	await NEXT_HMR();
	const [updatedStyles, updatedStyles2] = await Promise.all([
		import("./style.module.css"),
		import("./style2.module.css")
	]);
	expect(updatedStyles).toMatchObject({
		"class-other": "_style_module_css-class-other"
	});

	expect(updatedStyles2).toMatchObject({
		"bar": "_style2_module_css-bar"
	});
});
