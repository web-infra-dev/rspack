module.exports = {
  snapshotFileFilter(file) {
    return file.endsWith("main.mjs");
  },
  snapshotContent(content) {
    if (content.includes("unreachable")) {
      throw new Error("inactive bar dependency should not be rendered");
    }
    return content;
  },
};
