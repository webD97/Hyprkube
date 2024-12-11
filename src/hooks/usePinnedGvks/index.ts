import { useEffect, useRef, useState } from "react";
import { ClusterProfileId } from "../../api/listClusterProfiles";
import { Gvk } from "../../model/k8s";
import listPinnedGvks from "../../api/listPinnedGvks";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

interface PinnedGvksChangedEvent {
    clusterProfile: ClusterProfileId,
    gvks: Gvk[]
}

export default function usePinnedGvks(profile: ClusterProfileId | undefined) {
    const unlistenFn = useRef<UnlistenFn>();
    const [pinnedGvks, setPinnedGvks] = useState<Gvk[]>([]);

    useEffect(() => {
        if (!profile) {
            return;
        }

        listPinnedGvks(profile)
            .then(result => {
                setPinnedGvks(result)
            })
            .catch(e => {
                console.log("error listening")
                alert("Error listing pinned GVKs: " + JSON.stringify(e))
            });

        listen<PinnedGvksChangedEvent>('hyprkube://pinned-gvks-changed', (event) => {
            console.log({ event })
            if (event.payload.clusterProfile === profile) {
                setPinnedGvks(event.payload.gvks);
            }
        })
            .then((unlisten) => {
                unlistenFn.current = unlisten;
            })
            .catch(e => {
                alert("Error listening for changes to pinned GVKs: " + JSON.stringify(e))
            });

        return () => {
            if (unlistenFn.current) {
                unlistenFn.current();
            }
        }
    }, [profile]);

    return pinnedGvks;
}