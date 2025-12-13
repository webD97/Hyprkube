import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export async function deleteResource(
    contextSource: KubeContextSource,
    gvk: Gvk,
    namespace: string,
    name: string
) {
    return invoke('delete_resource', { contextSource, gvk, namespace, name, dryRun: false });
}