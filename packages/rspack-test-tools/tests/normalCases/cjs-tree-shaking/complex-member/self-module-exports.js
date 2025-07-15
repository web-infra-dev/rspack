module.exports.aaa = [
	{
		name: "a",
		index: 0
	},
	{
		name: "b",
		index: 1
	}
];

var aaa = {};

for (const k of module.exports.aaa) {
	aaa[k.name] = module.exports.aaa[k.index].name;
}

module.exports.abc = aaa;
