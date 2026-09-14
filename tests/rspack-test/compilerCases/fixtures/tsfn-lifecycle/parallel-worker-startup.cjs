require("./worker-startup.cjs")(true).catch(error => {
  console.error(error);
  process.exitCode = 1;
});
