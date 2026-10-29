import React, { useLayoutEffect, useMemo, useRef, useState } from 'react';

import { useVirtualizer } from '@tanstack/react-virtual';
import { App, Button, Divider, Input, Switch, theme, Tooltip } from 'antd';
import { LuChevronsDown, LuClock, LuCopy, LuHistory, LuSearch } from 'react-icons/lu';
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

interface ToggleProps {
    icon: React.ReactNode,
    label: string,
    tooltip: string,
    checked: boolean,
    onChange: (checked: boolean) => void,
}

const Toggle: React.FC<ToggleProps> = ({ icon, label, tooltip, checked, onChange }) => (
    <Tooltip title={tooltip}>
        <label className={classes.toggle}>
            <Switch size="small" checked={checked} onChange={onChange} />
            {icon}
            {label}
        </label>
    </Tooltip>
);

const LogPanel: React.FC<LogPanelProps> = (props) => {
    const {
        contextSource, namespace, name, container
    } = props;

    const parentRef = useRef(null);
    const { token } = theme.useToken();
    const { message } = App.useApp();

    const [search, setSearch] = useState('');
    const [follow, setFollow] = useState(true);
    const [timestamps, setTimestamps] = useState(false);
    const [previous, setPrevious] = useState(false);

    const text = usePodLogs(contextSource, namespace, name, container, { timestamps, previous });
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
            <div
                className={classes.toolbar}
                style={{
                    background: token.colorBgContainer,
                    borderBottom: `1px solid ${token.colorBorderSecondary}`,
                    fontFamily: token.fontFamily,
                    color: token.colorText,
                }}
            >
                <Input
                    size="small"
                    allowClear
                    className={classes.filter}
                    prefix={<LuSearch style={{ color: token.colorTextTertiary }} />}
                    placeholder="Filter lines"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                />
                {search && <span style={{ color: token.colorTextTertiary }}>{lines.length} matches</span>}
                <div className={classes.spacer} />
                <Toggle icon={<LuChevronsDown />} label="Follow" tooltip="Scroll to new lines as they arrive" checked={follow} onChange={setFollow} />
                <Toggle icon={<LuClock />} label="Timestamps" tooltip="Prefix each line with its timestamp" checked={timestamps} onChange={setTimestamps} />
                <Toggle icon={<LuHistory />} label="Previous" tooltip="Show logs of the previous container instance" checked={previous} onChange={setPrevious} />
                <Divider orientation="vertical" />
                <Button
                    size="small"
                    icon={<LuCopy />}
                    onClick={() => {
                        navigator.clipboard.writeText(text)
                            .then(() => message.success('Logs copied to clipboard'))
                            .catch(() => message.error('Could not copy logs'));
                    }}
                >
                    Copy
                </Button>
            </div>
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
                                minWidth: '100%',
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
