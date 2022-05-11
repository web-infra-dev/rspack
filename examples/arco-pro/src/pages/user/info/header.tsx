import React from 'react';
import { Avatar, Space, Skeleton } from '@arco-design/web-react';
import {
  IconCamera,
  IconLocation,
  IconUser,
  IconHome,
} from '@arco-design/web-react/icon';
import styles from './style/index.module.less';

interface HeaderProps {
  userInfo?: {
    name?: string;
    avatar?: string;
    jobName?: string;
    organizationName?: string;
    locationName?: string;
  };
  loading?: boolean;
}

function UserInfoHeader(props: HeaderProps) {
  const { userInfo = {}, loading } = props;

  const loadingNode = (
    <Skeleton
      text={{
        rows: 1,
        style: { width: '100px', height: '20px', marginBottom: '-4px' },
        width: ['100%'],
      }}
      animation
    />
  );
  const loadingImgNode = (
    <Skeleton
      text={{ rows: 0 }}
      image={{ style: { width: '64px', height: '64px' }, shape: 'circle' }}
      animation
    />
  );
  return (
    <div className={styles.header}>
      <Space
        size={8}
        direction="vertical"
        align="center"
        className={styles['header-content']}
      >
        {loading ? (
          loadingImgNode
        ) : (
          <Avatar size={64} triggerIcon={<IconCamera />}>
            <img src={userInfo.avatar} />
          </Avatar>
        )}
        <div className={styles.username}>
          {loading ? loadingNode : userInfo.name}
        </div>
        <div className={styles['user-msg']}>
          <Space size={18}>
            <div>
              <IconUser />
              <span className={styles['user-msg-text']}>
                {loading ? loadingNode : userInfo.jobName}
              </span>
            </div>
            <div>
              <IconHome />
              <span className={styles['user-msg-text']}>
                {loading ? loadingNode : userInfo.organizationName}
              </span>
            </div>
            <div>
              <IconLocation />
              <span className={styles['user-msg-text']}>
                {loading ? loadingNode : userInfo.locationName}
              </span>
            </div>
          </Space>
        </div>
      </Space>
    </div>
  );
}

export default UserInfoHeader;
