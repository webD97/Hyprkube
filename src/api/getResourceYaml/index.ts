import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";

export default function getResourceYaml(
    clientId: string,
    gvk: Gvk,
    namespace: string,
    name: string
) {
    return invoke<string>('get_resource_yaml', {
        clientId, gvk, namespace, name
    })
}