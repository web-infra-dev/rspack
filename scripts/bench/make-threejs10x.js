import "zx/globals";

await import("../meta/check_is_workspace_root.js");

await fs.ensureDir("./benchcases/threejs10x/src");

for (const i in Array(10).fill(null)) {
	await fs.ensureDir(`./benchcases/threejs10x/src/copy${i}`);

	await $`cp -r ./benchcases/.three/src ./benchcases/threejs10x/src/copy${i}`;
}

const entryCode = Array(10)
	.fill(null)
	.map((_, i) => i)
	.map(
		i => `import * as copy${i} from './copy${i}/Three.js'\nexport { copy${i} }`
	)
	.join("\n");

fs.writeFile("./benchcases/threejs10x/src/entry.js", entryCode);

// Create test.config.json

const testConfig = `
{
    "entry": {
        "index": {
            "import": ["./src/entry.js"]
        }
    }
}
`;

fs.writeFile("./benchcases/threejs10x/test.config.json", testConfig);

// Create  webpack.config.js

const webpackConfig = `
module.exports = {
    mode: 'development',
    entry: {
        index: ['./benchcases/three/src/entry.js']
    },
    devtool: 'eval',
    cache: {type: 'filesystem'}
}
`;

fs.writeFile("./benchcases/threejs10x/webpack.config.js", webpackConfig);
