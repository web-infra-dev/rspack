import React, { CSSProperties, ReactNode } from 'react';
import { Typography } from '@arco-design/web-react';
import cs from 'classnames';
import styles from './style/index.module.less';

interface PanelProps {
  className?: string;
  style?: CSSProperties;
  title?: ReactNode;
  children?: ReactNode;
}

function Panel(props: PanelProps) {
  const { className, style, title, children } = props;
  return (
    <div className={cs(styles.panel, className)} style={style}>
      <Typography.Title>{title}</Typography.Title>
      {children}
    </div>
  );
}

export default Panel;
