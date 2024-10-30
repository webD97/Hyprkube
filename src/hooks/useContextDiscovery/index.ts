import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export type KubeContextSource = [string, string];

export const useContextDiscovery = () => {
    const [sources, setSources] = useState<KubeContextSource[]>([]);

    useEffect(() => {
        (invoke("discover_contexts") as Promise<KubeContextSource[]>)
            .then(sources => setSources(sources))
            .catch(e => alert(e));
    }, []);

    return sources;
};
