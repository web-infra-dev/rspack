var globalName = "$GLOBAL$";
var uniqueName = $UNIQUE_NAME$;
var global = __webpack_require__.g;
if (global[globalName] && typeof global[globalName] !== "object") {
  console.warn("[RSPACK] Expose global failed: Global variable `" + globalName + "` already exists and its type is not `Object`.");
} else {
  var scope = global[globalName] = global[globalName] || {};
  if (typeof scope[uniqueName] !== 'undefined') {
    console.warn("[RSPACK] Expose global failed: Unique name `" + uniqueName + "` already exists.");
  } else {
    __webpack_require__.rg = scope[uniqueName] = scope[uniqueName] || {};
  }
}
