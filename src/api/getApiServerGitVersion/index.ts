import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export type CachedApiserverVersion = {
    gitVersion: string,
    fetchedAt: string,
};

export default async function getApiServerGitVersion(contextSource: KubeContextSource) {
    return invoke<CachedApiserverVersion | null>("get_apiserver_gitversion", { contextSource });
}
