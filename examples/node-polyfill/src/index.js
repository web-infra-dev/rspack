const util = require("util");
console.log("procss:", process.version);
console.log("buffer", Buffer.from("abcd"));
console.log("util:", util.format("%s:%s", "foo", "bar"));
