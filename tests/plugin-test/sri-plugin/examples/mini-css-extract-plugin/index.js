import("./style.module.css").then((module) => {
  console.log(module["test"] ? "ok" : "error");
});
