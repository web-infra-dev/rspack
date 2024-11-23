(() => { // webpackBootstrap
"use strict";
// check if unignored files are cleaned
const fs = require("fs")
const path = require("path")

it("should keep files", () => {
	expect(fs.existsSync(path.resolve(__dirname, "test-dist/should-not-be-removed"))).toBe(true)
});

it("should remove files", () => {
	expect(fs.existsSync(path.resolve(__dirname, "test-dist/removed.js"))).toBe(false)
});

})()
;