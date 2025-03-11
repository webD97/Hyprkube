import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export type KubeContextSource = {
    provider: string,
    source: string,
    context: string
};

export const useContextDiscovery = () => {
    const [sources, setSources] = useState<KubeContextSource[]>([]);

    useEffect(() => {
        invoke<KubeContextSource[]>("discover_contexts")
            .then(sources => setSources(sources))
            .catch(e => alert(e));
    }, []);

    return sources;
};
