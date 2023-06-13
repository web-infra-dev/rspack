const fs = require("fs");
const path = require("path");
const walk = require("acorn-walk");
const acorn = require("acorn");

const rspack = fs
	.readFileSync(path.resolve(__dirname, "./tree-shaking/ts-target-es5/expected/main.js"))
	.toString();
// const webpack = fs.readFileSync(path.resolve(__dirname, "./angular-webpack.js")).toString();

// console.log(file)

// console.log(file.search('"__esModule",{value:!0}'))
const testDir = path.resolve(__dirname, "./tree-shaking/")

const list = fs.readdirSync(testDir)

const res = {}
for (let i = 0; i < list.length; i++) {
	const item = list[i]
	if (item.startsWith(".") || item === "node_modules") {
		continue;
	}
	try {
		const dirabpath = path.join(testDir, item)
		const expectedfilePath = path.join(dirabpath, "expected/main.js")
		const file = fs.readFileSync(expectedfilePath).toString();
		const exportinfo = getExportInfo(file, true)
		res[item] = exportinfo
	} catch(err) {
		console.log(item)
		console.error(err)
	}
}

console.log(JSON.stringify(res))

function getExportInfo(file, isWebpack) {
	const exportInfo = {};
	let modulesNode = null;
	let len = 0;
	walk.simple(acorn.parse(file, { ecmaVersion: "latest" }), {
		ObjectExpression(node) {
			if (node.end - node.start > len) {
				modulesNode = node;
				len = node.end - node.start;
			}
		},
	});

	let properties = modulesNode.properties;
	for (let prop of properties) {
		const request = prop.key.value;
		let e;
		if (isWebpack) {
			e = getExportFromWebpack(prop.value);
		} else {
			e = getExportFromRspack(prop.value);
		}
		exportInfo[request] = e;
	}
	return exportInfo;
}

// const webpackExportInfo = getExportInfo(webpack, true)
// const rspackExportInfo = getExportInfo(rspack, true);
// console.log(rspackExportInfo)

function getExportFromRspack(func) {
  let exports = [];
	let imports = [];
  walk.simple(func, {
    CallExpression(node) {
      let callee = node.callee;
      if (
        callee.type === "MemberExpression" &&
        callee.object.name === "Object" &&
        callee.property.name === "defineProperty"
      ) {
        const arguments = node.arguments;
        if (arguments[0].type === "Identifier" && arguments[0].name === "exports") {
          exports.push(arguments[1].value);
        }
      }
      if (
        callee.type === "Identifier" &&
        callee.name === "_export" &&
        node.arguments[0].type === "Identifier" &&
        node.arguments[0].name === "exports"
      ) {
        const obj = node.arguments[1];
        let properties = obj.properties;
        for (let prop of properties) {
          exports.push(prop.key.name);
        }
      }

      if (
        callee.type === "Identifier" &&
        callee.name === "__webpack_require__"
      ) {
        const obj = node.arguments[0];
				if (obj.type === "Literal") {
					imports.push(obj.value)
				}
      }
    },
  });
	return {
		exports: exports.filter(item => item !== "__esModule"),
		imports
	}
}

function getExportFromWebpack(func) {
  let exports = [];
	let imports = []
  walk.simple(func, {
    CallExpression(node) {
      let callee = node.callee;
      if (
        callee.type === "MemberExpression" &&
        callee.object.name === "__webpack_require__" &&
        callee.property.name === "d"
      ) {
        const arguments = node.arguments;
        if (arguments[0].type === "Identifier" && arguments[0].name === "exports") {
          const obj = node.arguments[1];
          let properties = obj.properties;
          for (let prop of properties) {
            exports.push(prop.key.value);
          }
        }
      }

      if (
        callee.type === "Identifier" &&
        callee.name === "__webpack_require__"
      ) {
        const obj = node.arguments[0];
				if (obj.type === "Literal") {
					imports.push(obj.value)
				}
      }
    },
  });
  return {
		exports,
		imports
	}
}
