import "zx/globals";

await import("../meta/check_is_workspace_root.js");

const testConfig = `
{
	"devtool": "source-map",
	"builtins": {
		"minifyOptions": {
			"passes": 1,
			"dropConsole": true,
			"pureFuncs": []
		}
	},
	"optimization": {
		"moduleIds": "deterministic"
	},
	"entry": {
			"index": {
					"import": ["./src/entry.js"]
			}
	}
}
`;

fs.writeFile("./benchcases/threejs10x/test.config.json", testConfig);
