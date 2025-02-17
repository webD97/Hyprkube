import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";
import { ClusterProfileId } from "../listClusterProfiles";

export default function addPinnedGvk(profile: ClusterProfileId, gvk: Gvk) {
    return invoke<void>('cluster_profile_add_pinned_gvk', {
        profile,
        gvk
    });
}