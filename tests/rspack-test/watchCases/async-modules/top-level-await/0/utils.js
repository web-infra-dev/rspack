function escapeRegExp(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function isAsyncModule(content, moduleId) {
  const regex = new RegExp(`\\"${escapeRegExp(moduleId)}\\".*\\(.*\\).*\\{\\s([\\S\\s]*?)__webpack_require__\\.r\\(__webpack_exports__\\);`)
  const result = regex.exec(content)
  try {
    const [, header] = result;
    return header.trim().startsWith("__webpack_require__.a(")
  } catch (e) {
    console.log(content, moduleId, result)
    throw e;
  }
}

export function hasAsyncModuleRuntime(content) {
  const comment = "// " + ["webpack", "runtime", "async_module"].join("/");
  return content.includes(comment)
}
