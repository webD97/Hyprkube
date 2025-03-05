import { invoke } from "@tauri-apps/api/core";

export default function setDefaultNamespace(profile: string, namespace: string) {
    return invoke<void>('set_default_namespace', {
        profile, namespace
    })
}