const { graphql } = require("react-relay");

it("graphql", () => {
	// https://github.com/web-infra-dev/rspack/issues/2440
	// This will transformed to require('./custom/MyComponent.graphql.ts'),
	// If succeed, `require` would become  `__webpack_require__`,
	// Then it will resolved to mock.js
	const mock = graphql`
		fragment MyComponent on Type {
			field
		}
	`;

	expect(mock.found).toBeTruthy();
});
