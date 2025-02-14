import(/* webpackChunkName: "corrupt" */ "./corrupt")
  .then(function error() {
    console.log("error");
  })
  .catch(function ok() {
    console.log("ok");
  });
