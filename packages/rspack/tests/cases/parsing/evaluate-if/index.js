import { foo } from "./foo";

// see issue https://github.com/web-infra-dev/rspack/issues/5430
it("should compile", () => {
	if (typeof require === "undefined") {
		foo;
	}
});
