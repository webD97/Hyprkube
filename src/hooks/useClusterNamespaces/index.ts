import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Gvk } from "../../model/k8s";

export type WatchEvent =
    | {
        event: 'created';
        data: string
    }
    | {
        event: 'updated';
        data: string
    }
    | {
        event: 'deleted';
        data: string
    }

export default function useClusterNamespaces(kubernetesClientId: string | undefined, gvk: Gvk | undefined): string[] {
    const [namespaces, setNamespaces] = useState<string[]>([]);

    useEffect(() => {
        if (gvk === undefined) return;
        if (kubernetesClientId === undefined) return;

        const channel = new Channel<WatchEvent>();

        channel.onmessage = (message) => {
            if (message.event === 'created') {
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
            invoke('cleanup_channel', { channel });
        };
    }, [gvk, kubernetesClientId]);

    return namespaces;
}
