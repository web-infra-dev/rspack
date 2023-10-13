import { Container } from "./Button";

it("styled components", () => {
	console.log(Container.componentStyle);
	expect(Container.displayName).toMatch("Button__Container");
	expect(Container.styledComponentId).toMatch(
		/^Button__Container-rspack-test__/
	);
});
