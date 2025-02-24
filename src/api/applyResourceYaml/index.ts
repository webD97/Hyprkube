import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";

export default function applyResourceYaml(
    clientId: string,
    gvk: Gvk,
    namespace: string,
    name: string,
    newYaml: string
) {
    return invoke<string>('apply_resource_yaml', {
        clientId, gvk, namespace, name, newYaml
    })
}