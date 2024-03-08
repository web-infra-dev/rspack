module.exports = {
  findBundle(index) {
    return index === 1 ? ["main.js"] : []
  }
}