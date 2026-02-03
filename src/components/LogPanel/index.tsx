import React, { useLayoutEffect, useMemo, useRef, useState } from 'react';

import { useVirtualizer } from '@tanstack/react-virtual';
import { Button, Checkbox, Flex, Input } from 'antd';
import { KubeContextSource } from '../../hooks/useContextDiscovery';
import { usePodLogs } from '../../hooks/usePodLogs';
import Ansi from '../Ansi';
import classes from './component.module.css';

export interface LogPanelProps {
    contextSource: KubeContextSource,
    namespace: string,
    name: string
    container: string,
}

const LogPanel: React.FC<LogPanelProps> = (props) => {
    const {
        contextSource, namespace, name, container
    } = props;

    const parentRef = useRef(null);

    const [search, setSearch] = useState('');
    const [follow, setFollow] = useState(true);

    const text = usePodLogs(contextSource, namespace, name, container);
    const lines = useMemo(() => text.split('\n').filter(line => line.includes(search)), [search, text]);

    const rowVirtualizer = useVirtualizer({
        count: lines.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 21,
    });

    useLayoutEffect(() => {
        if (follow && lines.length > 0) {
            rowVirtualizer.scrollToOffset(Number.MAX_SAFE_INTEGER);
        }
    }, [follow, lines, rowVirtualizer]);

    return (
        <div className={classes.container}>
            <Flex gap="middle" align="center" justify="flex-end">
                <Button onClick={() => void navigator.clipboard.writeText(text)}>Copy to clipboard</Button>
                <Checkbox checked={follow} onChange={(e) => setFollow(e.target.checked)}>Follow</Checkbox>
                <Input type="search" style={{ width: "300px" }} placeholder="Filter lines" value={search} onChange={(e) => setSearch(e.target.value)} />
            </Flex>
            <div ref={parentRef} style={{ overflow: 'scroll', width: '100%', height: '100%', flexGrow: 1 }} onWheel={() => setFollow(false)}>
                <div className={classes.logWrapper}
                    style={{
                        height: `${rowVirtualizer.getTotalSize()}px`,
                        minHeight: '100%'
                    }}
                >
                    {rowVirtualizer.getVirtualItems().map((virtualItem) => (
                        <div
                            key={virtualItem.key}
                            style={{
                                position: 'absolute',
                                top: 0,
                                left: 0,
                                width: '100%',
                                transform: `translateY(${virtualItem.start}px)`,
                            }}
                        >
                            <pre><Ansi linkify>{lines[virtualItem.index]}</Ansi></pre>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}

export default LogPanel;
