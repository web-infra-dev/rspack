__webpack_require__.consumesLoadingData.initialConsumes.forEach(function(id) {
  __webpack_require__.m[id] = function(module) {
    // Handle case when module is used sync
    installedModules[id] = 0;
    delete __webpack_require__.c[id];
    var factory = resolveHandler(__webpack_require__.consumesLoadingData.moduleIdToConsumeDataMapping[id])();
    if (typeof factory !== "function")
      throw new Error(
        "Shared module is not available for eager consumption: " + id
      );
    module.exports = factory();
  };
});
