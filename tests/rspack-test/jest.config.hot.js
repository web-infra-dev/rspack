const config = require("./jest.config");

/** @type {import('jest').Config} */
module.exports = process.env.WASM ? { testPathIgnorePatterns: [".*"], passWithNoTests: true } : {
	...config,
	testMatch: ["<rootDir>/*.hottest.js"]
};
