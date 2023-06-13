const path = require("path");
const fs = require("fs");
const { diff } = require("jest-diff");

const beforeJson = fs.readFileSync(path.resolve(__dirname, "before.json"));
const afterJson = fs.readFileSync(path.resolve(__dirname, "after.json"));

const before = JSON.parse(beforeJson);
const after = JSON.parse(afterJson);

console.log(diff(before, after, {
	expand: false
}));
