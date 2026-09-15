module.exports = {
	snapshotContent(content) {
		expect(content).toContain(
			'import { createRequire } from "node:module";',
		);
		return content;
	},
};
