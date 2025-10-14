import * as style from "./style.css";

it("should compile and load style on demand", async () => {
	expect(style).toEqual(nsObj({}));
	await import("./style2.css").then(x => {
		expect(x).toEqual(nsObj({}));
	});
});
