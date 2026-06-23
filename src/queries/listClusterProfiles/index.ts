import { queryOptions } from "@tanstack/react-query";
import listClusterProfiles from "../../api/listClusterProfiles";

export default function listClusterProfilesQuery() {
    return queryOptions({
        queryKey: ['listClusterProfiles'],
        queryFn: () => listClusterProfiles(),
    });
}