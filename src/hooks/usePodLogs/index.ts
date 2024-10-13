import { Channel, invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { LogStreamEvent } from "../../api/LogStreamEvent";
import { KubernetesClient } from "../../model/k8s";

export const usePodLogs = (kubernetesClient: KubernetesClient|undefined, namespace: string, name: string) => {
    const [text, setText] = useState('');

    useEffect(() => {
        if (!kubernetesClient) return; 

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

        invoke('kube_stream_podlogs', { namespace, name, channel, clientId: kubernetesClient.id })
            .catch(e => setText(e));

        return () => {
            invoke('kube_stream_podlogs_cleanup', { channelId: channel.id });
        };
    }, [namespace, name, kubernetesClient]);

    return text;
};
