const fs = require("fs");
const { default: Ajv, _, Name } = require("ajv");
const path = require("path");
const standaloneCode = require("ajv/dist/standalone").default;
const terser = require("terser");

const configDir = path.resolve(__dirname, "../src/config");
const configSchema = path.resolve(configDir, "./schema.json");
const configCheck = path.resolve(configDir, "./schema.check.js");

const ajv = new Ajv({
	code: { source: true, optimize: true },
	messages: false,
	strictNumbers: false,
	logger: false
});

ajv.addKeyword({
	keyword: "instanceof",
	schemaType: "string",
	code(ctx) {
		const { data, schema } = ctx;
		ctx.fail(_`!(${data} instanceof ${new Name(schema)})`);
	}
});

ajv.addKeyword({
	keyword: "absolutePath",
	type: "string",
	schemaType: "boolean",

	code(ctx) {
		const { data, schema } = ctx;
		ctx.fail(
			_`${data}.includes("!") || (absolutePathRegExp.test(${data}) !== ${schema})`
		);
	}
});

ajv.removeKeyword("minLength");
ajv.addKeyword({
	keyword: "minLength",
	type: "string",
	schemaType: "number",

	code(ctx) {
		const { data, schema } = ctx;
		if (schema !== 1)
			throw new Error("Schema precompilation only supports minLength: 1");
		ctx.fail(_`${data}.length < 1`);
	}
});

ajv.removeKeyword("enum");
ajv.addKeyword({
	keyword: "enum",
	schemaType: "array",
	$data: true,

	code(ctx) {
		const { data, schema } = ctx;
		for (const item of schema) {
			if (typeof item === "object" && item !== null) {
				throw new Error(
					`Schema precompilation only supports primitive values in enum: ${JSON.stringify(
						item,
						null,
						2
					)}`
				);
			}
		}
		ctx.fail(
			schema.map(x => _`${data} !== ${x}`).reduce((a, b) => _`${a} && ${b}`)
		);
	}
});

const validate = ajv.compile(require(configSchema));
const code = standaloneCode(ajv, validate);
terser
	.minify(code, {
		compress: {
			passes: 3
		},
		mangle: true,
		ecma: 2015,
		toplevel: true
	})
	.then(minified => {
		const code =
			"/** This file was automatically generated, Run `pnpm precompile-schema` to update */\n" +
			minified.code;
		fs.promises.writeFile(configCheck, code);
	});
