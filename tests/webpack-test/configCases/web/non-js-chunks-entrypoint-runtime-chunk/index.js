import o from "other-package";
import p from "package";

it("should run", () => {
	console.log.bind(console, p, o);
});
