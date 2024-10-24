import { Channel, invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";

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

export const usePodLogs = (kubernetesClientId: string | undefined, namespace: string, name: string) => {
    const [text, setText] = useState('');

    useEffect(() => {
        console.log({ kubernetesClientId })
        if (!kubernetesClientId) return;

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

        invoke('kube_stream_podlogs', { namespace, name, channel, clientId: kubernetesClientId })
            .catch(e => setText(e));

        return () => {
            invoke('cleanup_channel', { channel });
        };
    }, [namespace, name, kubernetesClientId]);

    return text;
};
