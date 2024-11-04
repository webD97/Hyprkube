import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export type ClusterDiscovery = {
    clientId?: string,
    discovery: DiscoveryResult,
    lastError: string | undefined,
    loading: boolean
};

export type DiscoveredGroup = {
    name: string,
    isCrd: boolean,
    kinds: DiscoveredResource[]
}

export type DiscoveredResource = {
    version: string,
    kind: string,
    plural: string,
    views: string[]
}

export type DiscoveryResult = {
    gvks: { [key: string]: DiscoveredGroup },
}

export type AsyncDiscovery =
    {
        discoveredResource: [
            { group: string, version: string, kind: string, plural: string, source: 'Builtin' | 'CustomResource' },
            string[]
        ]
    };

export function useClusterDiscovery(source: string | null, context: string | null): ClusterDiscovery {
    const [clientId, setClientId] = useState<string>();
    const [discovery, setDiscovery] = useState<DiscoveryResult>({ gvks: {} });
    const [lastError, setLastError] = useState<string>();
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        if (source === null) return;
        if (context === null) return;

        const channel = new Channel<AsyncDiscovery>();

        channel.onmessage = (message) => {
            if ("discoveredResource" in message) {
                const [resource, views] = message.discoveredResource;

                setDiscovery((discovery) => {
                    const updated = { ...discovery };

                    if (!(resource.group in updated.gvks)) {
                        updated.gvks[resource.group] = {
                            name: resource.group,
                            isCrd: resource.source === 'CustomResource',
                            kinds: []
                        }
                    }

                    // Backend currently sends resources multiple times
                    if (updated.gvks[resource.group].kinds.findIndex(k => k.kind === resource.kind) === -1) {
                        updated.gvks[resource.group].kinds.push({
                            kind: resource.kind,
                            version: resource.version,
                            plural: resource.plural,
                            views
                        });
                    }

                    return updated;
                });
            }
        };

        listen<string>('ERR_CLUSTER_DISCOVERY', (e) => setLastError(e.payload))
            .then((unlisten) => {
                invoke<{ clientId: string }>('discover_kubernetes_cluster', { channel, contextSource: [source, context] })
                    .catch((e) => setLastError(e))
                    .then((response) => setClientId(response!.clientId))
                    .finally(() => {
                        setLoading(false);
                        unlisten();
                    });
            });

        return () => {
            invoke('cleanup_channel', { channel });
        };
    }, [context, source]);

    return {
        discovery, clientId, lastError, loading
    };
};
