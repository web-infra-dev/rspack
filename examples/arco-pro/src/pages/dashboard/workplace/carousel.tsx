import React from 'react';
import { Carousel } from '@arco-design/web-react';

const imageSrc = [
  '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/f7e8fc1e09c42e30682526252365be1c.jpg~tplv-uwbnlip3yd-webp.webp',
  '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/94e8dd2d6dc4efb2c8cfd82c0ff02a2c.jpg~tplv-uwbnlip3yd-webp.webp',
  '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/ec447228c59ae1ebe185bab6cd776ca4.jpg~tplv-uwbnlip3yd-webp.webp',
  '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/1d1580d2a5a1e27415ff594c756eabd8.jpg~tplv-uwbnlip3yd-webp.webp',
];
function C() {
  return (
    <Carousel
      indicatorType="slider"
      showArrow="never"
      autoPlay
      style={{
        width: '100%',
        height: 160,
      }}
    >
      {imageSrc.map((src, index) => (
        <div key={index}>
          <img
            src={src}
            style={{
              width: 280,
              transform: 'translateY(-30px)',
            }}
          />
        </div>
      ))}
    </Carousel>
  );
}

export default C;
