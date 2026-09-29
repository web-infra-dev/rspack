import fs from "node:fs";
import path from "node:path";

const expectedRemoteIds = [
	"webpack/container/remote/r07/value",
	"webpack/container/remote/r15/value"
];

function readRuntimeData(options) {
	const source = fs.readFileSync(
		path.join(options.output.path, options.output.filename),
		"utf-8"
	);
	const match = source.match(
		/remotesLoadingData\s*=\s*\{ chunkMapping: \{"main":(\[[^\]]+\])\}, moduleIdToRemoteDataMapping: \{(.*)\} \};/
	);

	expect(match).toBeTruthy();
	return {
		remoteIds: JSON.parse(match[1]),
		mappingKeys: [...match[2].matchAll(/"(webpack\/container\/remote\/[^"]+)":\{/g)].map(
			match => match[1]
		)
	};
}

export default {
	noTests: true,
	afterExecute(options) {
		const forward = readRuntimeData(options[0]);
		const reverse = readRuntimeData(options[1]);

		expect(forward.remoteIds).toEqual(expectedRemoteIds);
		expect(reverse.remoteIds).toEqual(expectedRemoteIds);
		expect(reverse.mappingKeys).toEqual(forward.mappingKeys);
	}
};
