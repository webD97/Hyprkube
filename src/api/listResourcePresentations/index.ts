import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export type ResourcePresentationDef = string;

export default async function listResourcePresentations(
    contextSource: KubeContextSource,
    gvk: Gvk,
): Promise<ResourcePresentationDef[]> {
    return invoke('list_resource_presentations', {
        ...gvk,
        contextSource,
    });
}
