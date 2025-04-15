import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export type WatchEvent =
    | {
        event: 'applied';
        data: string
    }
    | {
        event: 'deleted';
        data: string
    }

export default function useClusterNamespaces(kubernetesClientId: string | undefined): string[] {
    const [namespaces, setNamespaces] = useState<string[]>([]);

    useEffect(() => {
        if (kubernetesClientId === undefined) return;

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

        setNamespaces([]);

        invoke('watch_namespaces', { clientId: kubernetesClientId, channel })
            .catch(e => alert(e));

        return () => {
            void invoke('cleanup_channel', { channel });
        };
    }, [kubernetesClientId]);

    return namespaces;
}
