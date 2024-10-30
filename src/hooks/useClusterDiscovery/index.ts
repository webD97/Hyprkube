import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { KubeContextSource } from "../useContextDiscovery";

export type ClusterDiscovery = {
    clientId?: string,
    discovery: DiscoveryResult
};

export type DiscoveredGroup = {
    name: string,
    isCrd: boolean,
    kinds: DiscoveredResource[]
}

export type DiscoveredResource = {
    version: string,
    kind: string,
    views: string[]
}

export type DiscoveryResult = {
    gvks: { [key: string]: DiscoveredGroup },
}

export type AsyncDiscovery =
    {
        discoveredResource: [
            { group: string, version: string, kind: string, source: 'Builtin' | 'CustomResource' },
            string[]
        ]
    } |
    {
        apiGroup: [string, boolean]
    };

export function useClusterDiscovery(contextSource: KubeContextSource | undefined): ClusterDiscovery {
    const [clientId, setClientId] = useState<string>();
    const [discovery, setDiscovery] = useState<DiscoveryResult>({ gvks: {} });

    useEffect(() => {
        if (contextSource === undefined) return;

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
                            views
                        });
                    }

                    return updated;
                });
            }
        };

        (invoke('discover_kubernetes_cluster', { channel, contextSource }) as Promise<{ clientId: string }>)
            .then(({ clientId }) => setClientId(clientId))
            .catch(e => alert(e));

        return () => {
            invoke('cleanup_channel', { channel });
        };
    }, [contextSource]);

    return {
        discovery, clientId
    };
};
