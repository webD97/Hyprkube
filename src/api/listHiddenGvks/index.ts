import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";
import { ClusterProfileId } from "../listClusterProfiles";

export default function listHiddenGvks(profile: ClusterProfileId) {
    return invoke<Gvk[]>("cluster_profile_list_hidden_gvks", { profile })
}