import lower from "./lo";

const tests = {
	"simple template": () => require(`./langs/${lower("EN")}`),
	"double template": () => require(`./langs/${lower("E")}${lower("N")}`),
	"template with prefix": () => require(`./langs/${lower("EN")}.js`),
	"double template with prefix": () =>
		require(`./langs/${lower("E")}${lower("N")}.js`),
	"simple concat": () => require("./langs/".concat(lower("EN"))),
	"double concat": () => require("./langs/".concat(lower("E"), lower("N"))),
	"concat with prefix": () => require("./langs/".concat(lower("EN"), ".js")),
	"double concat with prefix": () =>
		require("./langs/".concat(lower("E"), lower("N"), ".js")),
	"simple plus": () => require("./langs/" + lower("EN")),
	"double plus": () => require("./langs/" + lower("E") + lower("N")),
	"plus with prefix": () => require("./langs/" + lower("EN") + ".js"),
	"double plus with prefix": () =>
		require("./langs/" + lower("E") + lower("N") + ".js")
};

for (const name of Object.keys(tests)) {
	it(`should handle imports in ${name} strings`, () => {
		expect(tests[name]().default).toBe("en");
	});
}
