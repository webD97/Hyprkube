import { invoke } from "@tauri-apps/api/core";
import { KubernetesClient } from "../model/k8s";

export function getDefaultKubernetesClient() {
    return invoke('initialize_kube_client') as Promise<KubernetesClient>;
}

export type DiscoveryResult = {
    gvks: { [key: string]: [string, string] }
}

export function discoverGvks(client: KubernetesClient): Promise<DiscoveryResult> {
    return invoke("kube_discover", { clientId: client.id });
}
