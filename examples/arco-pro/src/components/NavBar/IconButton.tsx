import React, { forwardRef } from 'react';
import { Button } from '@arco-design/web-react';
import styles from './style/icon-button.module.less';
import cs from 'classnames';

function IconButton(props, ref) {
  const { icon, className, ...rest } = props;

  return (
    <Button
      ref={ref}
      icon={icon}
      shape="circle"
      type="secondary"
      className={cs(styles['icon-button'], className)}
      {...rest}
    />
  );
}

export default forwardRef(IconButton);
