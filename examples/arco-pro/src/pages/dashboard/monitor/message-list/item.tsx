import React from 'react';
import { Space, Typography } from '@arco-design/web-react';
import { IconCommand, IconStar } from '@arco-design/web-react/icon';
import cs from 'classnames';
import styles from './style/index.module.less';

export interface Message {
  id?: string;
  username?: string;
  content?: string;
  time?: string;
  isCollect?: boolean;
}

export interface MessageItemProps {
  data: Message;
}

function MessageItem(props: MessageItemProps) {
  const { data = {} } = props;
  const classNames = cs(styles['message-item'], {
    [styles['message-item-collected']]: data.isCollect,
  });
  return (
    <div className={classNames}>
      <Space size={4} direction="vertical" style={{ width: '100%' }}>
        <Typography.Text type="warning">{data.username}</Typography.Text>
        <Typography.Text>{data.content}</Typography.Text>
        <div className={styles['message-item-footer']}>
          <div className={styles['message-item-time']}>
            <Typography.Text type="secondary">{data.time}</Typography.Text>
          </div>
          <div className={styles['message-item-actions']}>
            <div className={styles['message-item-actions-item']}>
              <IconCommand />
            </div>
            <div
              className={cs(
                styles['message-item-actions-item'],
                styles['message-item-actions-collect']
              )}
            >
              <IconStar />
            </div>
          </div>
        </div>
      </Space>
    </div>
  );
}

export default MessageItem;
