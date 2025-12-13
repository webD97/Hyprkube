import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { KubeContextSource } from "../useContextDiscovery";

export type LogStreamEvent =
    | {
        event: 'newLine',
        data: {
            lines: string[]
        }
    }
    | {
        event: 'endOfStream'
    }
    | {
        event: 'error'
        data: {
            msg: string
        }
    };

export const usePodLogs = (contextSource: KubeContextSource, namespace: string, name: string, container: string) => {
    const [text, setText] = useState('');

    useEffect(() => {
        // We really want to reset the state at this point:
        // eslint-disable-next-line react-hooks/set-state-in-effect
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

        invoke('kube_stream_podlogs', { namespace, name, channel, container, contextSource })
            .catch(e => setText(e as string));

        return () => {
            void invoke('cleanup_channel', { channel });
        };
    }, [namespace, name, contextSource, container]);

    return text;
};
