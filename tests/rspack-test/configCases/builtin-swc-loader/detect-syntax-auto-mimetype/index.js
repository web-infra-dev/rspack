import dataTs from "data:text/javascript;charset=utf-8;base64,Y29uc3QgbWVzc2FnZTogc3RyaW5nID0gImRhdGEtdHMiOyBleHBvcnQgZGVmYXVsdCBtZXNzYWdlOw==";
import dataTsx from "data:text/javascript;charset=utf-8;base64,Y29uc3QgZWxlbWVudCA9IDxkaXYgY2xhc3NOYW1lPSJkYXRhLXRzeCI+ZGF0YS10c3g8L2Rpdj47IGV4cG9ydCBkZWZhdWx0IGVsZW1lbnQ7";

it("should support javascript mimetype virtual modules together with ts and tsx files", () => {
	expect(dataTs).toBe("data-ts");
	expect(dataTsx.type).toBe("div");
	expect(dataTsx.props.className).toBe("data-tsx");
});
