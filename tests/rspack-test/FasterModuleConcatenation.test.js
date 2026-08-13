const path = require("path");
const {
	createConfigCase,
	createNormalCase,
	describeByWalk
} = require("@rspack/test-tools");

const rspackOptions = {
	experiments: {
		fasterModuleConcatenation: true
	}
};

function describeCases(caseType, caseGroup, createCase) {
	describeByWalk(
		__filename,
		(name, src, dist) => {
			createCase(`${caseType}/${name}`, src, dist, rspackOptions);
		},
		{
			source: path.resolve(__dirname, `${caseType}Cases/${caseGroup}`),
			dist: path.resolve(
				__dirname,
				`./js/faster-module-concatenation/${caseType}/${caseGroup}`
			),
			level: 1,
			describe: (name, fn) =>
				describe(`${caseType}/${caseGroup}/${name}`, fn)
		}
	);
}

describeCases("normal", "scope-hoisting", createNormalCase);
describeCases("config", "concatenate-modules", createConfigCase);
describeCases("config", "scope-hoisting", createConfigCase);
