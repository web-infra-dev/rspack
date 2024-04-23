const { graphql } = require("react-relay");

it("graphql", () => {
	const query = graphql`
		fragment MyComponent on Type {
			field
		}
	`;

	expect(query).toBe(1);
});
