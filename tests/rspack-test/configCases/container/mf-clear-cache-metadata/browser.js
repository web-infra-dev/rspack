it("omits native SSR cache code and reverse graph metadata from web output", () => {
  const load = () => import("./page");
  expect(typeof load).toBe("function");
  expect(__webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping).toBeTruthy();
  expect(__webpack_require__.remoteCache).toBeUndefined();
  expect(__webpack_require__.remotesLoadingData.remoteKeyToRemoteModuleIds).toBeUndefined();
  expect(__webpack_require__.remotesLoadingData.consumerModuleIdToParentModuleIds).toBeUndefined();
});
