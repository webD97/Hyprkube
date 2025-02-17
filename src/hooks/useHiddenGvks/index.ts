import { useEffect, useRef, useState } from "react";
import { ClusterProfileId } from "../../api/listClusterProfiles";
import { Gvk } from "../../model/k8s";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import listHiddenGvks from "../../api/listHiddenGvks";

interface HiddenGvksChangedEvent {
    clusterProfile: ClusterProfileId,
    gvks: Gvk[]
}

export default function useHiddenGvks(profile: ClusterProfileId | undefined) {
    const unlistenFn = useRef<UnlistenFn>(null);
    const [hiddenGvks, setHiddenGvks] = useState<Gvk[]>([]);

    useEffect(() => {
        if (!profile) {
            return;
        }

        listHiddenGvks(profile)
            .then(result => {
                setHiddenGvks(result)
            })
            .catch(e => {
                console.log("error listening")
                alert("Error listing hidden GVKs: " + JSON.stringify(e))
            });

        listen<HiddenGvksChangedEvent>('hyprkube://hidden-gvks-changed', (event) => {
            console.log({ event })
            if (event.payload.clusterProfile === profile) {
                setHiddenGvks(event.payload.gvks);
            }
        })
            .then((unlisten) => {
                unlistenFn.current = unlisten;
            })
            .catch(e => {
                alert("Error listening for changes to hidden GVKs: " + JSON.stringify(e))
            });

        return () => {
            if (unlistenFn.current) {
                unlistenFn.current();
            }
        }
    }, [profile]);

    return hiddenGvks;
}