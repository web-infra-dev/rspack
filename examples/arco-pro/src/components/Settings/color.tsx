import React from 'react';
import { Trigger, Typography } from '@arco-design/web-react';
import { SketchPicker } from 'react-color';
import { generate, getRgbStr } from '@arco-design/color';
import { useSelector, useDispatch } from 'react-redux';
import { GlobalState } from '../../store';
import useLocale from '@/utils/useLocale';
import styles from './style/color-panel.module.less';

function ColorPanel() {
  const theme =
    document.querySelector('body').getAttribute('arco-theme') || 'light';
  const settings = useSelector((state: GlobalState) => state.settings);
  const locale = useLocale();
  const themeColor = settings.themeColor;
  const list = generate(themeColor, { list: true });
  const dispatch = useDispatch();

  return (
    <div>
      <Trigger
        trigger="hover"
        position="bl"
        popup={() => (
          <SketchPicker
            color={themeColor}
            onChangeComplete={(color) => {
              const newColor = color.hex;
              dispatch({
                type: 'update-settings',
                payload: { settings: { ...settings, themeColor: newColor } },
              });
              const newList = generate(newColor, {
                list: true,
                dark: theme === 'dark',
              });
              newList.forEach((l, index) => {
                const rgbStr = getRgbStr(l);
                document.body.style.setProperty(
                  `--arcoblue-${index + 1}`,
                  rgbStr
                );
              });
            }}
          />
        )}
      >
        <div className={styles.input}>
          <div
            className={styles.color}
            style={{ backgroundColor: themeColor }}
          />
          <span>{themeColor}</span>
        </div>
      </Trigger>
      <ul className={styles.ul}>
        {list.map((item, index) => (
          <li
            key={index}
            className={styles.li}
            style={{ backgroundColor: item }}
          />
        ))}
      </ul>
      <Typography.Paragraph style={{ fontSize: 12 }}>
        {locale['settings.color.tooltip']}
      </Typography.Paragraph>
    </div>
  );
}

export default ColorPanel;
