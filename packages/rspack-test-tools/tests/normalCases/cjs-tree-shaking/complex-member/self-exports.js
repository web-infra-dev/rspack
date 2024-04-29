exports.aaa = [
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

for (let k of exports.aaa) {
	aaa[k.name] = exports.aaa[k.index].name;
}

exports.abc = aaa;
