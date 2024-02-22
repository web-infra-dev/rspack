// Check the mapping of various key locations back to the original source
export default async function checkSourceMap(out, outCodeMap, toSearch) {
	let failed = false;
	const recordCheck = (success, message) => {
		if (!success) {
			failed = true;
			console.error(`❌ ${message}`);
		}
	};

	const sourceMap = require("source-map");
	const path = require("path");

	const sources = JSON.parse(outCodeMap).sources;
	for (let source of sources) {
		if (sources.filter(s => s === source).length > 1) {
			throw new Error(
				`Duplicate source ${JSON.stringify(source)} found in source map`
			);
		}
	}
	const map = await new sourceMap.SourceMapConsumer(outCodeMap);
	for (const id in toSearch) {
		const outIndex = out.indexOf(id);
		if (outIndex < 0)
			throw new Error(`Failed to find "${id}" in output ${out}`);
		const outLines = out.slice(0, outIndex).split("\n");
		const outLine = outLines.length;
		const outLastLine = outLines[outLines.length - 1];
		let outColumn = outLastLine.length;
		const { source, line, column } = map.originalPositionFor({
			line: outLine,
			column: outColumn
		});

		const inSource = toSearch[id];
		recordCheck(
			source === inSource,
			`expected source: ${inSource}, observed source: ${source}@${line}:${column}, {out_source}@${outLine}:${outColumn}.`
		);

		const inCode = map.sourceContentFor(source);
		let inIndex = inCode.indexOf(id);
		if (inIndex < 0) inIndex = inCode.indexOf(`'${id}'`);
		if (inIndex < 0)
			throw new Error(`Failed to find "${id}" in input ${inCode}`);
		const inLines = inCode.slice(0, inIndex).split("\n");
		const inLine = inLines.length;
		const inLastLine = inLines[inLines.length - 1];
		let inColumn = inLastLine.length;

		if (path.extname(source) === "css") {
			const outMatch = /\s*content:\s*$/.exec(outLastLine);
			const inMatch = /\bcontent:\s*$/.exec(inLastLine);
			if (outMatch) outColumn -= outMatch[0].length;
			if (inMatch) inColumn -= inMatch[0].length;
		}

		const expected = JSON.stringify({ source, line: inLine, column: inColumn });
		const observed = JSON.stringify({ source, line, column });
		recordCheck(
			expected === observed,
			`expected original position: ${expected}, observed original position: ${observed}, out: ${
				outLine + "," + outColumn + "," + outIndex + ":" + id
			}`
		);

		// Also check the reverse mapping
		const positions = map.allGeneratedPositionsFor({
			source,
			line: inLine,
			column: inColumn
		});
		recordCheck(
			positions.length > 0,
			`expected generated positions: 1, observed generated positions: ${positions.length}`
		);
		let found = false;
		for (const { line, column } of positions) {
			if (line === outLine && column === outColumn) {
				found = true;
				break;
			}
		}
		const expectedPosition = JSON.stringify({
			line: outLine,
			column: outColumn
		});
		const observedPositions = JSON.stringify(positions);
		recordCheck(
			found,
			`expected generated position: ${expectedPosition}, observed generated positions: ${observedPositions}`
		);
	}

	return !failed;
}
