function loadStyles(url) {
  return new Promise((rsl, rej) => {
    var link = document.createElement('link')
    link.rel = 'stylesheet'
    link.type = 'text/css'
    link.href = url
    link.onload = rsl
    link.onerror = rej
    var head = document.getElementsByTagName('head')[0]
    head.appendChild(link)
  })
}
const ensurers = {
  async js(chunk_id) {
    await import('http://127.0.01:4444/' + chunk_id + '.js')
  },
  async css(chunk_id) {
    try {
      await loadStyles('http://127.0.01:4444/' + chunk_id + '.css')
    } catch (err) {
      console.log('css load fail', err)
    }
  },
}
function ensure(chunkId) {
  return Promise.all(
    Object.keys(ensurers).map((ensurerName) => {
      return ensurers[ensurerName](chunkId)
    }),
  )
}

async function __rspack_dynamic_require__(module_id, chunk_id) {
  await ensure(chunk_id)
  return globalThis.rs.require(module_id)
}

globalThis.rs.dynamic_require = globalThis.rs.dynamic_require || __rspack_dynamic_require__
