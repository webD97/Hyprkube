import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export type ClusterDiscovery = {
    discovery: DiscoveryResult,
    lastError: string | undefined,
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
    scope: 'namespaced' | 'cluster',
}

export type DiscoveryResult = {
    gvks: { [key: string]: DiscoveredGroup },
}

type DiscoveryResource = { group: string, version: string, kind: string, plural: string, source: 'Builtin' | 'CustomResource', scope: 'cluster' | 'namespaced' };

export type AsyncDiscovery =
    {
        discoveredResource: DiscoveryResource,
    }
    | {
        removedResource: DiscoveryResource,
    };

export type UseClusterDiscoveryOptions = {
    onStart?: () => void,
    onFinished?: () => void
}

export function useClusterDiscovery(source: string | null, context: string | null, { onStart, onFinished }: UseClusterDiscoveryOptions): ClusterDiscovery {
    const [discovery, setDiscovery] = useState<DiscoveryResult>({ gvks: {} });
    const [lastError, setLastError] = useState<string>();

    useEffect(() => {
        if (source === null) return;
        if (context === null) return;

        const channel = new Channel<AsyncDiscovery>();

        channel.onmessage = (message) => {
            if ("discoveryComplete" in message) {
                onFinished?.();
            }
            else if ("removedResource" in message) {
                const toBeRemoved = message.removedResource;

                setDiscovery((discovery) => {
                    if (!(toBeRemoved.group in discovery.gvks)) {
                        return discovery;
                    }

                    const gvks = { ...discovery.gvks };
                    gvks[toBeRemoved.group].kinds = gvks[toBeRemoved.group].kinds.filter(({ kind, version }) => kind !== toBeRemoved.kind || version !== toBeRemoved.version);

                    return ({ ...discovery, gvks });
                });
            } else if ("discoveredResource" in message) {
                const resource = message.discoveredResource;

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
                        const x = {
                            kind: resource.kind,
                            version: resource.version,
                            plural: resource.plural,
                            scope: resource.scope,
                        };

                        updated.gvks[resource.group].kinds.push(x);
                    }

                    return updated;
                });
            }
        };

        onStart?.();
        invoke<void>('connect_cluster', { channel, contextSource: { provider: 'file', source, context } })
            .catch((e) => {
                if (e === 'BackgroundTaskRejected') return;
                setLastError(e as unknown as string);
            });

        return () => {
            void invoke('cleanup_channel', { channel });
        };
    }, [context, onFinished, onStart, source]);

    return {
        discovery, lastError
    };
};
