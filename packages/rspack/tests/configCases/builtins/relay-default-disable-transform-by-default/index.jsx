const { graphql } = require("react-relay");

it("graphql should be used even with `disableTransformByDefault` is on", () => {
	const query = graphql`
		fragment MyComponent on Type {
			field
		}
	`;

	expect(query).toBe(1);
});
