import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export default async function listPodContainerNames(
    contextSource: KubeContextSource,
    namespace: string,
    name: string
): Promise<string[]> {
    return invoke('list_pod_container_names', { contextSource, namespace, name });
}