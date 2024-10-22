import { invoke } from "@tauri-apps/api/core";
import { KubernetesClient } from "../model/k8s";

export function getDefaultKubernetesClient() {
    return invoke('initialize_kube_client') as Promise<KubernetesClient>;
}

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
    crdApigroups: string[],
    builtinApigroups: string[]
}

export function discoverGvks(client: KubernetesClient): Promise<DiscoveryResult> {
    return invoke("kube_discover", { clientId: client.id });
}
