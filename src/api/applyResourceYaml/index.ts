import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export default function applyResourceYaml(
    contextSource: KubeContextSource,
    gvk: Gvk,
    namespace: string,
    name: string,
    newYaml: string
) {
    return invoke<string>('apply_resource_yaml', {
        contextSource, gvk, namespace, name, newYaml
    })
}