import React from 'react';

const element = <div>
  <style jsx>{`
    div {
      color: red;
    }
  `}</style>
</div>;

it("has jsx- className", () => {
  // scoped css class should be added like jsx-7b844396f2efe2ef
  expect(element.props.className).toMatch(/jsx-(.+)/);
});
