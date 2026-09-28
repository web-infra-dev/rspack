it('should remove the nested block when the issuer changes', () => new Promise((resolve, reject) => {
  require.ensure([], () => {
    expect(require('./b')).toBe(43);
    resolve();
  }, error => reject(error));
}));
