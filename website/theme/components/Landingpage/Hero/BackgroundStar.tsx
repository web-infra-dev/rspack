import { useEffect, useRef, useState } from 'react';
import styles from './index.module.scss';

const BackgroundStar = ({
  top,
  left,
  pageX,
  pageY,
  size,
}: {
  top: number | string;
  left: number | string;
  pageX: number | null;
  pageY: number | null;
  size: number;
}) => {
  const ref = useRef<any>();
  const [transformX, setTransformX] = useState<number>(0);
  const [transformY, setTransformY] = useState<number>(0);
  useEffect(() => {
    if (ref.current) {
      const { x, y } = ref.current.getBoundingClientRect();
      const { width, height } = ref.current.getBoundingClientRect();
      const { width: windowWidth, height: windowHeight } =
        document.body.getBoundingClientRect();
      const { width: starWidth, height: starHeight } =
        ref.current.getBoundingClientRect();

      const bodyScrollTop =
        document.body.scrollTop ||
        document.getElementsByTagName('html')[0].scrollTop;
      const bodyScrollLeft = document.body.scrollLeft;
    }
  }, []);

  return (
    <div
      className={styles.backgroundStarContainer}
      ref={ref}
      style={{
        top,
        left,
        transform: `translate(${transformX}px, ${transformY}px)`,
      }}
    >
      <svg
        className={styles.backgroundStar}
        style={{ width: size, height: size }}
        xmlns="http://www.w3.org/2000/svg"
        width="8"
        height="9"
        viewBox="0 0 8 9"
        fill="none"
        role="img"
        aria-label="star"
      >
        <title />
        <circle cx="4" cy="4.5" r="4" fill="url(#paint0_radial_2202_5618)" />
        <defs>
          <radialGradient
            id="paint0_radial_2202_5618"
            cx="0"
            cy="0"
            r="1"
            gradientUnits="userSpaceOnUse"
            gradientTransform="translate(4 4.49998) scale(3.77871 4.29149)"
          >
            <stop stopColor="#FF8B00" />
            <stop offset="0.38" stopColor="#F2A65A" />
            <stop offset="0.59" stopColor="#FFB966" />
            <stop offset="0.92" stopColor="#FF8B00" />
          </radialGradient>
        </defs>
      </svg>
    </div>
  );
};

export default BackgroundStar;
