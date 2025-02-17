import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";
import { ClusterProfileId } from "../listClusterProfiles";

export default function addHiddenGvk(profile: ClusterProfileId, gvk: Gvk) {
    return invoke<void>('cluster_profile_add_hidden_gvk', {
        profile,
        gvk
    });
}