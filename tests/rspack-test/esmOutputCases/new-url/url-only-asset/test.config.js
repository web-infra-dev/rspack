module.exports = {
  snapshotFileFilter(file) {
    return file.endsWith('.mjs') || file.endsWith('.txt')
  },
}
