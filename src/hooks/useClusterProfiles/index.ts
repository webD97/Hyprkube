import { useEffect, useState } from "react";
import listClusterProfiles, { ClusterProfile } from "../../api/listClusterProfiles";

export default function useClusterProfiles() {
    const [clusterProfiles, setClusterProfiles] = useState<ClusterProfile[]>([]);

    useEffect(() => {
        listClusterProfiles()
            .then(profiles => {
                setClusterProfiles(profiles);
            })
            .catch(e => alert(JSON.stringify(e)));
    }, []);

    return clusterProfiles;
}