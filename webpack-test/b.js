const process = require('process')
let data = ""
let error = ""


process.stdin.on("readable", () => {
  let chunk;
  while (null !== (chunk = process.stdin.read())) {
    data += chunk;
  }
});

process.stdin.on("end", () => {
  // process all the data and write it back to stdout
	console.log(data)
});

