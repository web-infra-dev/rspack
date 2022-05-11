import React, { ReactNode } from 'react';
import { Switch, Divider, InputNumber } from '@arco-design/web-react';
import { useSelector, useDispatch } from 'react-redux';
import { GlobalState } from '../../store';
import useLocale from '../../utils/useLocale';
import styles from './style/block.module.less';

export interface BlockProps {
  title?: ReactNode;
  options?: { name: string; value: string; type?: 'switch' | 'number' }[];
  children?: ReactNode;
}

export default function Block(props: BlockProps) {
  const { title, options, children } = props;
  const locale = useLocale();
  const settings = useSelector((state: GlobalState) => state.settings);
  const dispatch = useDispatch();

  return (
    <div className={styles.block}>
      <h5 className={styles.title}>{title}</h5>
      {options &&
        options.map((option) => {
          const type = option.type || 'switch';

          return (
            <div className={styles['switch-wrapper']} key={option.value}>
              <span>{locale[option.name]}</span>
              {type === 'switch' && (
                <Switch
                  size="small"
                  checked={!!settings[option.value]}
                  onChange={(checked) => {
                    const newSetting = {
                      ...settings,
                      [option.value]: checked,
                    };
                    dispatch({
                      type: 'update-settings',
                      payload: { settings: newSetting },
                    });
                    // set color week
                    if (checked && option.value === 'colorWeek') {
                      document.body.style.filter = 'invert(80%)';
                    }
                    if (!checked && option.value === 'colorWeek') {
                      document.body.style.filter = 'none';
                    }
                  }}
                />
              )}
              {type === 'number' && (
                <InputNumber
                  style={{ width: 80 }}
                  size="small"
                  value={settings.menuWidth}
                  onChange={(value) => {
                    const newSetting = {
                      ...settings,
                      [option.value]: value,
                    };
                    dispatch({
                      type: 'update-settings',
                      payload: { settings: newSetting },
                    });
                  }}
                />
              )}
            </div>
          );
        })}
      {children}
      <Divider />
    </div>
  );
}
