export function getFooMock() {
  return rs.requireMock('../src/foo');
}

globalThis.__keepAlive = [getFooMock];
