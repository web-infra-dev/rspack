import { Container } from "./Button";
import { Container as Container2 } from "./Button2";

it("styled components", () => {
	expect(Container.styledComponentId).toMatch(
		/^Button__Container-rspack-test__/
	);
	expect(Container2.styledComponentId).toMatch(
		/^Button2__Container-rspack-test__/
	);
});
