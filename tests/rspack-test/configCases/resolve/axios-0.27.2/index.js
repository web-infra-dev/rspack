import axios from "axios-demo";

it("should resolve xhr when default", () => {
	// `browser_field` should enabled
	expect(axios.adapter).toBe("xhr");
});
