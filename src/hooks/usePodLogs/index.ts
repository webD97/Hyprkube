import { Channel, invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { LogStreamEvent } from "../../api/LogStreamEvent";

export const usePodLogs = (namespace: string, name: string) => {
    const [text, setText] = useState('');

    useEffect(() => {
        setText('');

        const channel = new Channel<LogStreamEvent>();

        channel.onmessage = (message) => {
            if (message.event === 'newLine') {
                setText(text => text + message.data.lines.join('\n'));
            }
            else if (message.event === 'endOfStream') {
                setText(text => text + '(end of stream)');
            }
            else if (message.event === 'error') {
                setText(text => text + 'stream error: ' + message.data.msg);
            }
        };

        invoke('kube_stream_podlogs', { namespace, name, channel })
            .catch(e => setText(e));

        return () => {
            invoke('kube_stream_podlogs_cleanup', { channelId: channel.id });
        };
    }, [namespace, name]);

    return text;
};
