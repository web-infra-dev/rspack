import css from './style.less';

it('keeps plugin variable tables valid across calls and object operations', () => {
  expect(css).toBe(`.root {
  value: 42px;
}
.first {
  saved: 1;
}
.second {
  saved: 2;
}
.saved {
  first: 11px;
  second: 22px;
}
.inspect {
  result: ok;
}
`);
});
