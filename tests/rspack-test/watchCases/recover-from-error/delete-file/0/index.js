it('should recover when recreate file', function () {
  switch (WATCH_STEP) {
    case '0':
      expect(require('./file')).toBe('ok');
      break;
    case '1':
      expect(function () {
        require('./file');
      }).toThrow();
      break;
    case '2':
      expect(require('./file')).toBe('ok');
      break;
  }
});
