this.aaa = [
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

for (let k of this.aaa) {
	aaa[k.name] = this.aaa[k.index].name;
}

this.abc = aaa;
