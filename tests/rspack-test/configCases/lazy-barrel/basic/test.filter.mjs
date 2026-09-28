// Node v16 doesn't support readdir recursive :(
export default () => !process.version.startsWith("v16")
