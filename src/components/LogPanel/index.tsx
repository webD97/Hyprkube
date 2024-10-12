import React from 'react';

import classes from './component.module.css';
import { LazyLog, ScrollFollow } from '@melloware/react-logviewer';
import { usePodLogs } from '../../hooks/usePodLogs';

export interface LogPanelProps {
    namespace: string,
    name: string
}

const LogPanel: React.FC<LogPanelProps> = (props) => {
    const {
        namespace, name
    } = props;

    const text = usePodLogs(namespace, name);

    return (
        <div className={classes.container}>
            <ScrollFollow
                startFollowing={true}
                render={({ follow, onScroll }) => (
                    <LazyLog
                        text={text}
                        enableLineNumbers={false}
                        enableSearch
                        caseInsensitive
                        follow={follow}
                        onScroll={onScroll}
                    />
                )}
            />
        </div>
    );
}

export default LogPanel;
