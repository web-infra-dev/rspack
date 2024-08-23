import styles from './index.module.scss';

const BackgroundStar = ({
  top,
  left,
  size,
}: {
  top: number | string;
  left: number | string;
  size: number;
}) => {
  return (
    <div
      className={styles.backgroundStarContainer}
      style={{
        top,
        left,
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
            <stop stop-color="#FF8B00" />
            <stop offset="0.38" stop-color="#F2A65A" />
            <stop offset="0.59" stop-color="#FFB966" />
            <stop offset="0.92" stop-color="#FF8B00" />
          </radialGradient>
        </defs>
      </svg>
    </div>
  );
};

export default BackgroundStar;
