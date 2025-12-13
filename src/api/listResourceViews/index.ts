import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export type ResourceViewDef = string;

export default async function listResourceViews(
    contextSource: KubeContextSource,
    gvk: Gvk,
): Promise<ResourceViewDef[]> {
    return invoke('list_resource_views', {
        ...gvk,
        contextSource,
    });
}
