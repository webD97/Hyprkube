import { queryOptions } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export type GroupedContextSources = Record<string, {
    contexts: KubeContextSource[]
}>;

export default function discoverContextsQuery() {
    return queryOptions({
        queryKey: ['discover_contexts'],
        queryFn: () => invoke<KubeContextSource[]>("discover_contexts"),
        select(data) {
            const groupedContextSources: GroupedContextSources = {};

            data.forEach((contextSource) => {
                const { provider, source } = contextSource;
                let displayName = source;

                if (source.includes('Lens/')) {
                    displayName = source.substring(0, source.lastIndexOf('/'));
                }

                displayName = provider + "://" + displayName;

                if (!(displayName in groupedContextSources)) {
                    groupedContextSources[displayName] = {
                        contexts: []
                    };
                }

                groupedContextSources[displayName].contexts.push(contextSource);
            });

            return groupedContextSources;
        },
    });
}