import { invoke } from "@tauri-apps/api/core";

export default function getDefaultNamespace(profile: string) {
    return invoke<string>('get_default_namespace', {
        profile
    })
}