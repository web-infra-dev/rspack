import fs from "node:fs";
import path from "node:path";

const expectedRemoteIds = [
	"webpack/container/remote/r07/value",
	"webpack/container/remote/r15/value",
	"webpack/container/remote/r23/value",
	"webpack/container/remote/r31/value",
	"webpack/container/remote/r39/value",
	"webpack/container/remote/r47/value"
];

function readRuntimeData(options) {
	const source = fs.readFileSync(
		path.join(options.output.path, options.output.filename),
		"utf-8"
	);
	const match = source.match(
		/remotesLoadingData\s*=\s*\{ chunkMapping: (\{.*\}), moduleIdToRemoteDataMapping: (\{.*\}) \};/
	);

	expect(match).toBeTruthy();
	return {
		runtimeSource: match[0],
		chunkMapping: JSON.parse(match[1]),
		moduleIdToRemoteDataMapping: JSON.parse(match[2])
	};
}

function expectMeaningfulRuntimeData(data) {
	expect(Object.keys(data.chunkMapping)).toHaveLength(5);
	expect(Object.values(data.chunkMapping).flat().sort()).toEqual(expectedRemoteIds);
	expect(Object.keys(data.moduleIdToRemoteDataMapping).sort()).toEqual(expectedRemoteIds);
}

export default {
	noTests: true,
	afterExecute(options) {
		const forward = readRuntimeData(options[0]);
		const reverse = readRuntimeData(options[1]);

		expectMeaningfulRuntimeData(forward);
		expectMeaningfulRuntimeData(reverse);
		expect(reverse.runtimeSource).toBe(forward.runtimeSource);
	}
};
