this.abc = {};

for (const i of [
	{
		name: "a"
	},
	{
		name: "b"
	}
]) {
	this.abc[i.name] = i.name;
}
