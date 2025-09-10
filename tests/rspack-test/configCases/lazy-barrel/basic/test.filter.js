// Node v16 doesn't support readdir recursive :(
module.exports = () => !process.version.startsWith("v16")
