/** @type {import("@rspack/core").LoaderDefinition<string>} */
module.exports = function () {
	const { name, expect, usedExports } = JSON.parse(this.query.slice(1));
	return [
		`if (Math.random() < 0) require(${JSON.stringify(
			`../_helpers/testModuleLoader?${JSON.stringify(usedExports)}!`
		)});`,
		"",
		...Object.keys(expect).map((source, i) =>
			[
				`import { __usedExports as usedExports_${i} } from ${JSON.stringify(
					source
				)};`,
				`it("${name} should have the correct exports used for ${source}", () => {`,
				`const expectedUsedExports = ${JSON.stringify(expect[source])};`,
				`const normalizedExpectedUsedExports = Array.isArray(expectedUsedExports)`,
				`\t? expectedUsedExports.concat(["__usedExports"]).sort()`,
				`\t: expectedUsedExports;`,
				`const actualUsedExports = Array.isArray(usedExports_${i})`,
				`\t? [...usedExports_${i}].sort()`,
				`\t: usedExports_${i};`,
				`const allowedExpectedUsedExports = [normalizedExpectedUsedExports];`,
				`// Production-side reexport retargeting may bypass the synthetic wrapper`,
				`// module used by this helper and leave only its local marker export active.`,
				`if (Array.isArray(expectedUsedExports)) {`,
				`\tallowedExpectedUsedExports.push(["__usedExports"]);`,
				`}`,
				`expect(allowedExpectedUsedExports).toContainEqual(actualUsedExports);`,
				`});`,
				""
			].join("\n")
		)
	].join("\n");
};
