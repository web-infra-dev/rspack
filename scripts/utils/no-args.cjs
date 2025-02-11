const { execSync } = require("node:child_process");
const cmd = process.argv[2];

execSync(cmd);
