import React from 'react';
import { Typography, Badge } from '@arco-design/web-react';
import styles from './style/index.module.less';

const { Text } = Typography;
interface TooltipProps {
  title: string;
  data: {
    name: string;
    value: string;
    color: string;
  }[];
  color?: string;
  name?: string;
  formatter?: (value: string) => React.ReactNode;
}

function CustomTooltip(props: TooltipProps) {
  const { formatter = (value) => value, color, name } = props;
  return (
    <div className={styles['customer-tooltip']}>
      <div className={styles['customer-tooltip-title']}>
        <Text bold>{props.title}</Text>
      </div>
      <div>
        {props.data.map((item, index) => (
          <div className={styles['customer-tooltip-item']} key={index}>
            <div>
              <Badge color={color || item.color} />
              {name || item.name}
            </div>
            <div>
              <Text bold>{formatter(item.value)}</Text>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default CustomTooltip;
