const fs = require("fs");
const path = require("path");
const { intersection, retain, each, includedIn, not } = require("./math.cjs");

async function _recursiveCompareBase(
	rootA,
	baseA,
	baseB,
	onCompare,
	identical = new Set(),
	difference = new Set()
) {
	const a = fs.readdirSync(baseA);
	const b = fs.readdirSync(baseB);
	let set = new Set(intersection(a, b));
	await Promise.all(
		Array.from(set).map(async item => {
			if (item === "node_modules" && rootA === baseA) {
				return Promise.resolve();
			}
			let nextA = path.join(baseA, item);
			let nextB = path.join(baseB, item);
			let fileA = fs.lstatSync(nextA).isFile();
			let fileB = fs.lstatSync(nextB).isFile();
			let p = path.relative(rootA, nextA);

			if (fileA && fileB) {
				let a = fs.readFileSync(nextA);
				let b = fs.readFileSync(nextB);

				let r = await onCompare(p, a, b);
				if (r) {
					identical.add(p);
				} else {
					difference.add(p);
				}
			} else if (fileA || fileB) {
				difference.add(p);
			} else {
				await _recursiveCompareBase(
					rootA,
					nextA,
					nextB,
					onCompare,
					identical,
					difference
				);
			}
		})
	);
}

async function recursiveCompare(
	baseA,
	baseB,
	onCompare,
	identical = new Set(),
	difference = new Set()
) {
	await _recursiveCompareBase(
		baseA,
		baseA,
		baseB,
		onCompare,
		identical,
		difference
	);
	return [identical, difference];
}

const exclude = arr =>
	retain(each(not(includedIn(["dist", "test.filter.js"]))))(arr);

async function recursiveCompareStrict(baseA, baseB, onCompare) {
	const a = exclude(fs.readdirSync(baseA));
	const b = exclude(fs.readdirSync(baseB));
	if (a.length !== b.length) {
		return false;
	}
	a.sort();
	b.sort();
	if (a.toString() !== b.toString()) {
		return false;
	}

	return (
		await Promise.all(
			a.map(async item => {
				let nextA = path.join(baseA, item);
				let nextB = path.join(baseB, item);
				let fileA = fs.lstatSync(nextA).isFile();
				let fileB = fs.lstatSync(nextB).isFile();
				if (fileA && fileB) {
					let a = fs.readFileSync(nextA);
					let b = fs.readFileSync(nextB);
					return onCompare(nextA, a, b);
				} else if (fileA || fileB) {
					return false;
				}
				return recursiveCompareStrict(nextA, nextB, onCompare);
			})
		)
	).every(Boolean);
}

module.exports = {
	recursiveCompare,
	recursiveCompareStrict
};
