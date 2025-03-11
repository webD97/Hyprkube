import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";

export default function getDefaultNamespace(profile: string, gvk: Gvk) {
    return invoke<string>('get_default_namespace', {
        profile, gvk
    })
}