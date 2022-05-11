import React from 'react';
import styles from '../style/blocks.module.less';
import { Card, Avatar, Typography, Skeleton } from '@arco-design/web-react';
const { Text, Title } = Typography;

export interface ProjectProps {
  title?: string;
  enTitle?: string;
  contributors?: {
    name?: string;
    email?: string;
    avatar?: string;
  }[];
  contributorsLength?: number;
  loading?: boolean;
}

function ProjectCard(props: ProjectProps) {
  const { loading, contributors } = props;
  return (
    <Card className={styles['project-wrapper']} bordered={true} size="small">
      {loading ? (
        <Skeleton text={{ rows: 1 }} animation />
      ) : (
        <Title heading={6}>{props.title}</Title>
      )}

      {loading ? (
        <Skeleton text={{ rows: 1 }} animation style={{ marginTop: '4px' }} />
      ) : (
        <Text type="secondary" ellipsis style={{ margin: '0' }}>
          {props.enTitle}
        </Text>
      )}

      <div className={styles.avatar}>
        {loading ? (
          <Skeleton text={{ rows: 1 }} animation />
        ) : (
          <>
            <Avatar.Group size={24}>
              {(contributors || []).map((item, index) => (
                <Avatar key={index}>
                  <img src={item.avatar} />
                </Avatar>
              ))}
            </Avatar.Group>
            <Text type="secondary" className={styles.more}>
              等{(props.contributorsLength || 0).toLocaleString()}人
            </Text>
          </>
        )}
      </div>
    </Card>
  );
}

export default ProjectCard;
