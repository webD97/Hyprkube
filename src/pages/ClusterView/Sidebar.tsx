import { ClusterProfileId } from "../../api/listClusterProfiles";
import GvkList from "../../components/GvkList";
import { DiscoveryResult } from "../../hooks/useClusterDiscovery";
import useHiddenGvks from "../../hooks/useHiddenGvks";
import usePinnedGvks from "../../hooks/usePinnedGvks";
import { Gvk } from "../../model/k8s";
import { createMenuForNormalGvks, createMenuForPinnedGvks } from "./menus";
import classes from './Sidebar.module.css';
;

export interface SidebarProps {
    clusterProfile: ClusterProfileId,
    discovery: DiscoveryResult,
    onGvkClicked: (gvk: Gvk, target: '_self' | '_blank') => void,
}

export default function Sidebar({
    clusterProfile,
    discovery,
    onGvkClicked
}: SidebarProps) {
    const pinnedGvks = usePinnedGvks(clusterProfile);
    const hiddenGvks = useHiddenGvks(clusterProfile);

    const sortedPinnedGvks = pinnedGvks.sort((a, b) => {
        if (a.kind < b.kind) {
            return -1;
        }
        if (a.kind > b.kind) {
            return 1;
        }

        return 0;
    });

    return (
        <nav className={classes.sidebarContainer}>
            <h2 className={classes.resourceSectionTitle}>Pinned resources</h2>
            {
                sortedPinnedGvks.length > 0 && (
                    <GvkList flat withGroupNames
                        gvks={sortedPinnedGvks}
                        onResourceClicked={(gvk) => onGvkClicked(gvk, '_self')}
                        onResourceAuxClicked={(gvk) => onGvkClicked(gvk, '_blank')}
                        onGvkContextMenu={(gvk) => createMenuForPinnedGvks({
                            clusterProfile,
                            openInNewTab: () => onGvkClicked(gvk, '_blank'),
                            gvk
                        })}
                    />
                )
            }

            <h2 className={classes.resourceSectionTitle}>Builtin resources</h2>
            {
                Object.values(discovery?.gvks || [])
                    .filter((group) => !group.isCrd)
                    .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                    .map(({ name: groupName, kinds }, idx) => {
                        const gvks = kinds
                            .map(({ kind, version }) => ({ group: groupName, version, kind }))
                            .filter(gvk => (
                                !hiddenGvks.some(current => (
                                    current.group === gvk.group &&
                                    current.version === gvk.version &&
                                    current.kind === gvk.kind
                                ))
                            ));

                        // Don't show groups where everything is hidden
                        if (gvks.length === 0) return;

                        return (
                            <GvkList key={idx}
                                gvks={gvks}
                                onResourceClicked={(gvk) => onGvkClicked(gvk, '_self')}
                                onResourceAuxClicked={(gvk) => onGvkClicked(gvk, '_blank')}
                                onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                    clusterProfile,
                                    openInNewTab: () => onGvkClicked(gvk, '_blank'),
                                    gvk
                                })}
                            />
                        );
                    })
            }

            <h2 className={classes.resourceSectionTitle}>Custom resources</h2>
            {
                Object.values(discovery?.gvks || [])
                    .filter((group) => group.isCrd)
                    .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                    .map(({ name: groupName, kinds }, idx) => {
                        const gvks = kinds
                            .map(({ kind, version }) => ({ group: groupName, version, kind }))
                            .filter(gvk => (
                                !hiddenGvks.some(current => (
                                    current.group === gvk.group &&
                                    current.version === gvk.version &&
                                    current.kind === gvk.kind
                                ))
                            ));

                        // Don't show groups where everything is hidden
                        if (gvks.length === 0) return;

                        return (
                            <GvkList key={idx}
                                gvks={gvks}
                                onResourceClicked={(gvk) => onGvkClicked(gvk, '_self')}
                                onResourceAuxClicked={(gvk) => onGvkClicked(gvk, '_blank')}
                                onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                    clusterProfile,
                                    openInNewTab: () => onGvkClicked(gvk, '_blank'),
                                    gvk
                                })}
                            />
                        );
                    })
            }
        </nav>
    );
}