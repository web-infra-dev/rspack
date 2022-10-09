import React from 'react';
import styles from './index.module.less'

export default function HelloWorld() {
  return (
    <div className={styles.hello}>
      Hello World, I am being styled using Less!
    </div>
  )
}
