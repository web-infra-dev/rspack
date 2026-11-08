export default {
  snapshotFileFilter(file) {
    return file.endsWith('main.mjs') || file.endsWith('asset.txt')
  },
}
