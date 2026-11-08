import path from "node:path";

export default function (source) {
	const dependency = path.resolve(import.meta.dirname, "dependency.txt");
	this.addDependency(dependency);
	this.callback(
		null,
		source,
		{
			version: 3,
			sources: [this.resourcePath],
			names: [],
			mappings: "",
		},
		{ marker: "preserved" },
	);
}
