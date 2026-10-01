import path from "node:path";
/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	const resolve1 = this.getResolve();
	const resolve2 = this.getResolve({
		extensions: [".xyz", ".js"]
	});
	const resolve3 = this.getResolve({
		extensions: [".hee", "..."]
	});
	const resolve4 = this.getResolve({
		extensions: [".xyz", "..."]
	});
	const resolve5 = this.getResolve({
		extensions: ["...", ".xyz"]
	});
	return Promise.all([
		resolve1(import.meta.dirname, "./index"),
		resolve2(import.meta.dirname, "./index"),
		resolve3(import.meta.dirname, "./index"),
		resolve4(import.meta.dirname, "./index"),
		resolve5(import.meta.dirname, "./index")
	]).then(([one, two, three, four, five]) => {
		return `module.exports = ${JSON.stringify({
			one: path.basename(one),
			two: path.basename(two),
			three: path.basename(three),
			four: path.basename(four),
			five: path.basename(five)
		})}`;
	});
};
