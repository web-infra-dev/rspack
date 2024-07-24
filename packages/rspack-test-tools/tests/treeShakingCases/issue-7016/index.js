if (true) {
  const hasConsole = typeof console !== 'undefined';
  const warn = (msg) => {
    if (hasConsole) {
      console.warn(msg)
    }
  }
  warn()
}
