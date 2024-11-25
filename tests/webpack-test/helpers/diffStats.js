const diff = require("jest-diff").diff;
const { stripVTControlCharacters: stripAnsi } = require("node:util");

const processStats = str => {
  return str.trim().split('\n').map(i => i.trim()).join('\n').replace(/\d+(\.\d+)?/g, 'XX').replace(/"/g, "");
};
const webpackStats = require('../__snapshots__/StatsTestCases.basictest.js.snap.webpack');

module.exports = (rawStats, name) => {
  const key = `StatsTestCases should print correct stats for ${name} 1`;
  const res = stripAnsi(
    diff(processStats(rawStats), processStats(webpackStats[key] || ''), { expand: false, contextLines: 0 })
  );
  return res;
};

