module.exports = async function () {
  return `export default "${JSON.parse(this.query.slice(1)).content}";`
}