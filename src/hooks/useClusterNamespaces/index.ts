import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { KubeContextSource } from "../useContextDiscovery";

export type WatchEvent =
    | {
        event: 'applied';
        data: string
    }
    | {
        event: 'deleted';
        data: string
    }

export default function useClusterNamespaces(contextSource: KubeContextSource): string[] {
    const [namespaces, setNamespaces] = useState<string[]>([]);

    useEffect(() => {
        const channel = new Channel<WatchEvent>();

        channel.onmessage = (message) => {
            if (message.event === 'applied') {
                setNamespaces(namespaces => ([
                    ...namespaces,
                    message.data
                ]));
            }
            else if (message.event === 'deleted') {
                setNamespaces(namespaces => {
                    return namespaces.filter(n => n !== message.data);
                });
            }
        };

        // We really want to reset the state at this point:
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setNamespaces([]);

        invoke('watch_namespaces', { contextSource, channel })
            .catch(e => {
                if (e === 'BackgroundTaskRejected') return;
                alert("blubb" + e);
            });

        return () => {
            void invoke('cleanup_channel', { channel });
        };
    }, [contextSource]);

    return namespaces;
}
