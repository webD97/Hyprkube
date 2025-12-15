import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export default function getResourceYaml(
    contextSource: KubeContextSource,
    gvk: Gvk,
    namespace: string,
    name: string
) {
    return invoke<string>('get_resource_yaml', {
        contextSource, gvk, namespace, name
    })
}