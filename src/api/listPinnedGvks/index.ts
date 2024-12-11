import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";
import { ClusterProfileId } from "../listClusterProfiles";

export default function listPinnedGvks(profile: ClusterProfileId) {
    return invoke<Gvk[]>("cluster_profile_list_pinned_gvks", { profile })
}