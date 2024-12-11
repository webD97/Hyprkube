import { invoke } from "@tauri-apps/api/core";

export type ClusterProfileId = string;
export type ClusterProfile = [ClusterProfileId, string]

export default function listClusterProfiles() {
    return invoke<ClusterProfile[]>('list_cluster_profiles');
}