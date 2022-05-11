import React, { useState, useEffect } from 'react';
import { Avatar, Typography, List, Skeleton } from '@arco-design/web-react';
import axios from 'axios';

const { Text } = Typography;

interface ITeam {
  avatar?: string;
  name?: string;
  members?: number;
}

function MyTeam() {
  const [data, setData] = useState<ITeam[]>(new Array(4).fill({}));
  const [loading, setLoading] = useState(true);

  const getData = async () => {
    const { data } = await axios
      .get('/api/users/teamList')
      .finally(() => setLoading(false));
    setData(data);
  };

  useEffect(() => {
    getData();
  }, []);

  return (
    <List
      dataSource={data}
      render={(item, index) => {
        return (
          <List.Item
            key={index}
            style={
              index !== data.length - 1
                ? { padding: '8px 0px' }
                : { padding: '8px 0px 0px 0px' }
            }
          >
            {loading ? (
              <Skeleton
                animation
                text={{ width: ['80%', '20%'], rows: 2 }}
                image={{ shape: 'circle' }}
              />
            ) : (
              <List.Item.Meta
                avatar={
                  <Avatar size={48}>
                    <img src={item.avatar} />
                  </Avatar>
                }
                title={item.name}
                description={
                  <Text type="secondary" style={{ fontSize: '12px' }}>{`共${(
                    item.members || 0
                  ).toLocaleString()}人`}</Text>
                }
              />
            )}
          </List.Item>
        );
      }}
    />
  );
}

export default MyTeam;
