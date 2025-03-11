import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";

export default function setDefaultNamespace(profile: string, gvk: Gvk, namespace: string) {
    return invoke<void>('set_default_namespace', {
        profile, gvk, namespace
    })
}