import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export default async function getApiServerGitVersion(contextSource: KubeContextSource) {
    return invoke<string>("get_apiserver_gitversion", { contextSource });
}
